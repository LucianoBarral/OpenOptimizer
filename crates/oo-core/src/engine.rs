use crate::journal::Journal;
use crate::model::*;
use oo_sys::SystemBackend;

pub fn new_batch_id() -> String {
    ulid::Ulid::new().to_string()
}

fn now() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

/// Motor de reversão: toda mudança passa por aqui, com o estado original
/// gravado no journal *antes* de tocar no sistema.
pub struct Engine<B: SystemBackend> {
    backend: B,
    journal: Journal,
}

impl<B: SystemBackend> Engine<B> {
    pub fn new(backend: B, journal: Journal) -> Self {
        Self { backend, journal }
    }

    pub fn backend(&self) -> &B {
        &self.backend
    }

    pub fn history(&self) -> Vec<Change> {
        self.journal.changes()
    }

    pub fn pending(&self) -> Vec<Change> {
        self.history()
            .into_iter()
            .filter(|c| c.status == ChangeStatus::Pending)
            .collect()
    }

    fn capture(&self, a: &Action) -> Result<Step, OoError> {
        Ok(match a {
            Action::RegSet { key, value } => Step {
                action: a.clone(),
                before: Before::Reg(self.backend.reg_get(key)?),
                after: After::Reg(Some(value.clone())),
            },
            Action::RegDelete { key } => Step {
                action: a.clone(),
                before: Before::Reg(self.backend.reg_get(key)?),
                after: After::Reg(None),
            },
            Action::PowerPlan { .. } => Step {
                action: a.clone(),
                before: Before::Power {
                    previous: self.backend.power_active()?,
                },
                after: After::Power { created: None },
            },
        })
    }

    fn execute(&self, step: &mut Step) -> Result<(), OoError> {
        match &step.action {
            Action::RegSet { key, value } => self.backend.reg_set(key, value)?,
            Action::RegDelete { key } => self.backend.reg_delete(key)?,
            Action::PowerPlan { source_guid } => {
                let g = self.backend.power_duplicate(source_guid)?;
                step.after = After::Power {
                    created: Some(g.clone()),
                };
                self.backend.power_set_active(&g)?;
            }
        }
        Ok(())
    }

    fn undo(&self, step: &Step) -> Result<(), OoError> {
        match (&step.action, &step.before, &step.after) {
            (Action::RegSet { key, .. } | Action::RegDelete { key }, Before::Reg(prev), _) => {
                match prev {
                    Some(v) => self.backend.reg_set(key, v)?,
                    None => self.backend.reg_delete(key)?,
                }
            }
            (Action::PowerPlan { .. }, Before::Power { previous }, After::Power { created }) => {
                self.backend.power_set_active(previous)?;
                if let Some(g) = created {
                    self.backend.power_delete(g)?;
                }
            }
            _ => return Err(OoError::Io("step inconsistente no journal".into())),
        }
        Ok(())
    }

    fn run_post(&self, post: Option<PostHook>) -> Result<(), OoError> {
        if let Some(PostHook::RefreshMouse) = post {
            self.backend.refresh_mouse()?;
        }
        Ok(())
    }

    pub fn apply(&mut self, t: &TweakDef, batch_id: &str) -> Result<Change, OoError> {
        let steps = t
            .actions
            .iter()
            .map(|a| self.capture(a))
            .collect::<Result<Vec<_>, _>>()?;
        let mut c = Change {
            id: ulid::Ulid::new().to_string(),
            tweak_id: t.id.clone(),
            batch_id: batch_id.into(),
            timestamp: now(),
            steps,
            status: ChangeStatus::Pending,
            post: t.post,
        };
        self.journal.record(&c)?; // write-ahead: originais no disco antes de qualquer alteração
        for i in 0..c.steps.len() {
            let mut step = c.steps[i].clone();
            let res = self.execute(&mut step);
            c.steps[i] = step;
            if let Err(e) = res {
                for done in c.steps[..i].iter().rev() {
                    let _ = self.undo(done);
                }
                c.status = ChangeStatus::Failed;
                self.journal.record(&c)?;
                return Err(e);
            }
        }
        self.run_post(c.post)?;
        c.status = ChangeStatus::Applied;
        self.journal.record(&c)?;
        Ok(c)
    }

    /// O valor atual ainda é o que nós escrevemos? Se não, outro programa mexeu.
    fn step_is_intact(&self, s: &Step) -> Result<bool, OoError> {
        Ok(match (&s.action, &s.after) {
            (Action::RegSet { key, .. } | Action::RegDelete { key }, After::Reg(expected)) => {
                &self.backend.reg_get(key)? == expected
            }
            (Action::PowerPlan { .. }, After::Power { created }) => {
                created.as_deref() == Some(self.backend.power_active()?.as_str())
            }
            _ => false,
        })
    }

    pub fn revert(&mut self, change_id: &str, force: bool) -> Result<(), OoError> {
        let mut c = self
            .history()
            .into_iter()
            .find(|c| c.id == change_id)
            .ok_or_else(|| OoError::NotFound(change_id.into()))?;
        if !force {
            for s in &c.steps {
                if !self.step_is_intact(s)? {
                    return Err(OoError::Conflict {
                        change_id: c.id.clone(),
                    });
                }
            }
        }
        for s in c.steps.iter().rev() {
            self.undo(s)?;
        }
        self.run_post(c.post)?;
        c.status = ChangeStatus::Reverted;
        c.timestamp = now();
        self.journal.record(&c)
    }

    fn active_change_for(&self, tweak_id: &str) -> Option<Change> {
        self.history()
            .into_iter()
            .rev()
            .find(|c| c.tweak_id == tweak_id && c.status == ChangeStatus::Applied)
    }

    pub fn revert_tweak(&mut self, tweak_id: &str, force: bool) -> Result<(), OoError> {
        let c = self
            .active_change_for(tweak_id)
            .ok_or_else(|| OoError::NotFound(tweak_id.into()))?;
        self.revert(&c.id, force)
    }

    pub fn revert_all(&mut self, force: bool) -> Vec<(String, OoError)> {
        let ids: Vec<String> = self
            .history()
            .into_iter()
            .rev()
            .filter(|c| matches!(c.status, ChangeStatus::Applied | ChangeStatus::Pending))
            .map(|c| c.id)
            .collect();
        ids.into_iter()
            .filter_map(|id| self.revert(&id, force).err().map(|e| (id, e)))
            .collect()
    }

    pub fn detect(&self, t: &TweakDef) -> TweakState {
        let mut hit = 0usize;
        for a in &t.actions {
            let ok = match a {
                Action::RegSet { key, value } => match self.backend.reg_get(key) {
                    Ok(v) => v.as_ref() == Some(value),
                    Err(_) => return TweakState::Unsupported,
                },
                Action::RegDelete { key } => matches!(self.backend.reg_get(key), Ok(None)),
                Action::PowerPlan { .. } => self.active_change_for(&t.id).is_some_and(|c| {
                    c.steps
                        .iter()
                        .any(|s| self.step_is_intact(s).unwrap_or(false))
                }),
            };
            if ok {
                hit += 1;
            }
        }
        match hit {
            0 => TweakState::NotApplied,
            n if n == t.actions.len() => TweakState::Applied,
            _ => TweakState::Partial,
        }
    }

    pub fn score(&self, catalog: &[TweakDef]) -> u8 {
        let (mut got, mut total) = (0u32, 0u32);
        for t in catalog {
            match self.detect(t) {
                TweakState::Unsupported => {}
                TweakState::Applied => {
                    got += t.weight;
                    total += t.weight;
                }
                _ => total += t.weight,
            }
        }
        // Sem tweaks suportados neste PC não há nada a melhorar: 100.
        (got * 100).checked_div(total).map_or(100, |s| s as u8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oo_sys::{FakeBackend, Hive, RegKey, RegValue, SystemBackend};

    fn key(n: &str) -> RegKey {
        RegKey::new(Hive::Hkcu, r"Software\T", n)
    }
    fn set(n: &str, v: u32) -> Action {
        Action::RegSet {
            key: key(n),
            value: RegValue::Dword(v),
        }
    }
    fn tweak(id: &str, actions: Vec<Action>) -> TweakDef {
        TweakDef {
            id: id.into(),
            category: "fps".into(),
            risk: Risk::Safe,
            requires_reboot: false,
            weight: 1,
            actions,
            post: None,
        }
    }
    fn eng() -> (Engine<FakeBackend>, tempfile::TempDir) {
        let d = tempfile::tempdir().unwrap();
        let j = Journal::open(d.path().join("j.jsonl")).unwrap();
        (Engine::new(FakeBackend::new(), j), d)
    }

    #[test]
    fn apply_then_revert_restores_previous_value() {
        let (mut e, _d) = eng();
        e.backend()
            .reg_set(&key("A"), &RegValue::Dword(20))
            .unwrap();
        let t = tweak("t", vec![set("A", 10)]);
        let c = e.apply(&t, "b1").unwrap();
        assert_eq!(
            e.backend().reg_get(&key("A")).unwrap(),
            Some(RegValue::Dword(10))
        );
        assert_eq!(e.detect(&t), TweakState::Applied);
        e.revert(&c.id, false).unwrap();
        assert_eq!(
            e.backend().reg_get(&key("A")).unwrap(),
            Some(RegValue::Dword(20))
        );
        assert_eq!(e.detect(&t), TweakState::NotApplied);
    }

    #[test]
    fn revert_of_previously_missing_value_deletes_it() {
        let (mut e, _d) = eng();
        let c = e.apply(&tweak("t", vec![set("A", 1)]), "b").unwrap();
        e.revert(&c.id, false).unwrap();
        assert_eq!(e.backend().reg_get(&key("A")).unwrap(), None);
    }

    #[test]
    fn failure_mid_tweak_rolls_back_its_steps() {
        let (mut e, _d) = eng();
        e.backend().fail_on_set("B");
        assert!(e
            .apply(&tweak("t", vec![set("A", 1), set("B", 1)]), "b")
            .is_err());
        assert_eq!(e.backend().reg_get(&key("A")).unwrap(), None);
        assert_eq!(e.history()[0].status, ChangeStatus::Failed);
    }

    #[test]
    fn external_change_is_a_conflict_unless_forced() {
        let (mut e, _d) = eng();
        let c = e.apply(&tweak("t", vec![set("A", 1)]), "b").unwrap();
        e.backend()
            .reg_set(&key("A"), &RegValue::Dword(99))
            .unwrap();
        assert!(matches!(
            e.revert(&c.id, false),
            Err(OoError::Conflict { .. })
        ));
        assert_eq!(
            e.backend().reg_get(&key("A")).unwrap(),
            Some(RegValue::Dword(99))
        );
        e.revert(&c.id, true).unwrap();
        assert_eq!(e.backend().reg_get(&key("A")).unwrap(), None);
    }

    #[test]
    fn power_plan_apply_and_revert() {
        let (mut e, _d) = eng();
        let orig = e.backend().power_active().unwrap();
        let t = tweak(
            "pp",
            vec![Action::PowerPlan {
                source_guid: "e9a42b02-d5df-448d-aa00-03f14749eb61".into(),
            }],
        );
        let c = e.apply(&t, "b").unwrap();
        assert_ne!(e.backend().power_active().unwrap(), orig);
        assert_eq!(e.detect(&t), TweakState::Applied);
        e.revert(&c.id, false).unwrap();
        assert_eq!(e.backend().power_active().unwrap(), orig);
        assert_eq!(e.detect(&t), TweakState::NotApplied);
    }

    #[test]
    fn pending_after_crash_is_reported_and_revertible() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("j.jsonl");
        let b = FakeBackend::new();
        // Simula crash: Pending gravado e valor já alterado, mas Applied nunca gravado.
        Journal::open(p.clone())
            .unwrap()
            .record(&Change {
                id: "c1".into(),
                tweak_id: "t".into(),
                batch_id: "b".into(),
                timestamp: "2026-11-27T10:00:00Z".into(),
                status: ChangeStatus::Pending,
                post: None,
                steps: vec![Step {
                    action: set("A", 1),
                    before: Before::Reg(Some(RegValue::Dword(5))),
                    after: After::Reg(Some(RegValue::Dword(1))),
                }],
            })
            .unwrap();
        b.reg_set(&key("A"), &RegValue::Dword(1)).unwrap();
        let mut e = Engine::new(b, Journal::open(p).unwrap());
        assert_eq!(e.pending().len(), 1);
        e.revert("c1", true).unwrap();
        assert_eq!(
            e.backend().reg_get(&key("A")).unwrap(),
            Some(RegValue::Dword(5))
        );
        assert!(e.pending().is_empty());
    }

    #[test]
    fn score_weights_applied_over_supported() {
        let (mut e, _d) = eng();
        let mut a = tweak("a", vec![set("A", 1)]);
        a.weight = 3;
        let b = tweak("b", vec![set("B", 1)]);
        assert_eq!(e.score(&[a.clone(), b.clone()]), 0);
        e.apply(&a, "x").unwrap();
        assert_eq!(e.score(&[a, b]), 75);
    }

    #[test]
    fn post_hook_runs_on_apply_and_revert() {
        let (mut e, _d) = eng();
        let mut t = tweak(
            "m",
            vec![Action::RegSet {
                key: key("M"),
                value: RegValue::Sz("0".into()),
            }],
        );
        t.post = Some(PostHook::RefreshMouse);
        let c = e.apply(&t, "b").unwrap();
        e.revert(&c.id, false).unwrap();
        assert_eq!(e.backend().mouse_refreshes(), 2);
    }
}
