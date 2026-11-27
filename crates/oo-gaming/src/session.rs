use crate::config::GamingConfig;
use crate::status::{GamingState, GamingStatus};
use crate::sys::{GamingSys, PersistedSession, SessionStore};
use crate::{PURGE_BELOW_MB, PURGE_MIN_INTERVAL_S};

#[derive(Debug)]
struct Active {
    exe: String,
    previous_scheme: Option<String>,
    last_tick: u64,
}

/// Máquina de estados do Modo Gaming. Não conhece Windows: recebe o sistema por trait.
#[derive(Debug, Default)]
pub struct Session {
    active: Option<Active>,
    last_purge: Option<u64>,
}

fn rfc3339(unix: u64) -> String {
    chrono::DateTime::from_timestamp(unix as i64, 0)
        .unwrap_or_default()
        .to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

impl Session {
    /// Ao iniciar: se uma sessão anterior foi interrompida (queda, desligamento),
    /// restaura o plano de energia gravado antes da troca.
    pub fn recover(sys: &dyn GamingSys, store: &mut dyn SessionStore) {
        if let Some(p) = store.load() {
            if let Some(prev) = p.previous_scheme {
                let _ = sys.set_scheme(&prev);
            }
            let _ = store.clear();
        }
    }

    /// Um passo do loop. Devolve `true` se o status mudou (para gravar o arquivo).
    pub fn tick(
        &mut self,
        cfg: &GamingConfig,
        sys: &dyn GamingSys,
        store: &mut dyn SessionStore,
        now: u64,
        status: &mut GamingStatus,
    ) -> bool {
        if !cfg.enabled {
            return self.deactivate(sys, store, now, status);
        }
        let running = sys.running_exes();
        let game = cfg.find_enabled(&running).map(|g| g.exe.to_lowercase());
        let current = self.active.as_ref().map(|a| a.exe.clone());
        match (current, game) {
            (None, None) => false,
            (None, Some(g)) => self.activate(&g, sys, store, now, status),
            (Some(_), None) => self.deactivate(sys, store, now, status),
            (Some(cur), Some(g)) if cur != g => {
                self.deactivate(sys, store, now, status);
                self.activate(&g, sys, store, now, status)
            }
            (Some(_), Some(_)) => self.maintain(sys, now, status),
        }
    }

    /// Daemon encerrando (Modo Gaming desligado): desfaz tudo.
    pub fn shutdown(
        &mut self,
        sys: &dyn GamingSys,
        store: &mut dyn SessionStore,
        now: u64,
        status: &mut GamingStatus,
    ) {
        self.deactivate(sys, store, now, status);
    }

    fn activate(
        &mut self,
        exe: &str,
        sys: &dyn GamingSys,
        store: &mut dyn SessionStore,
        now: u64,
        status: &mut GamingStatus,
    ) -> bool {
        let prev = sys.active_scheme().ok();
        let ultimate = sys.ensure_ultimate_scheme().ok();
        let previous_scheme = match (&prev, &ultimate) {
            (Some(p), Some(u)) if !p.eq_ignore_ascii_case(u) => Some(p.clone()),
            _ => None,
        };
        // Write-ahead: grava o plano anterior ANTES de trocar.
        let _ = store.save(&PersistedSession {
            game: exe.into(),
            previous_scheme: previous_scheme.clone(),
            started: now,
        });
        if let (Some(_), Some(u)) = (&previous_scheme, &ultimate) {
            let _ = sys.set_scheme(u);
        }
        let _ = sys.set_timer(true);
        self.active = Some(Active {
            exe: exe.into(),
            previous_scheme,
            last_tick: now,
        });
        status.state = GamingState::Active;
        status.game = Some(exe.into());
        status.since = Some(rfc3339(now));
        status.sessions += 1;
        true
    }

    fn maintain(&mut self, sys: &dyn GamingSys, now: u64, status: &mut GamingStatus) -> bool {
        let mut changed = false;
        if let Some(a) = self.active.as_mut() {
            status.seconds_in_game += now.saturating_sub(a.last_tick);
            a.last_tick = now;
        }
        let interval_ok = self
            .last_purge
            .is_none_or(|t| now.saturating_sub(t) >= PURGE_MIN_INTERVAL_S);
        if interval_ok && sys.free_mb() < PURGE_BELOW_MB {
            if let Ok(freed) = sys.purge_standby() {
                status.purges += 1;
                status.purged_mb += freed;
                changed = true;
            }
            self.last_purge = Some(now);
        }
        changed
    }

    fn deactivate(
        &mut self,
        sys: &dyn GamingSys,
        store: &mut dyn SessionStore,
        now: u64,
        status: &mut GamingStatus,
    ) -> bool {
        let Some(a) = self.active.take() else {
            return false;
        };
        status.seconds_in_game += now.saturating_sub(a.last_tick);
        let _ = sys.set_timer(false);
        if let Some(prev) = &a.previous_scheme {
            let _ = sys.set_scheme(prev);
        }
        let _ = store.clear();
        status.state = GamingState::Idle;
        status.game = None;
        status.since = None;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::GameEntry;
    use crate::status::GamingState;
    use crate::sys::{FakeGamingSys, MemStore, PersistedSession, FAKE_BALANCED, FAKE_ULTIMATE};

    fn cfg(games: &[(&str, bool)]) -> GamingConfig {
        GamingConfig {
            enabled: true,
            notify: true,
            games: games
                .iter()
                .map(|(e, on)| GameEntry {
                    exe: e.to_string(),
                    name: e.to_string(),
                    source: "manual".into(),
                    enabled: *on,
                })
                .collect(),
        }
    }

    struct Rig {
        sys: FakeGamingSys,
        store: MemStore,
        s: Session,
        st: GamingStatus,
    }
    fn rig() -> Rig {
        Rig {
            sys: FakeGamingSys::default(),
            store: MemStore::default(),
            s: Session::default(),
            st: GamingStatus::default(),
        }
    }
    impl Rig {
        fn tick(&mut self, c: &GamingConfig, now: u64) -> bool {
            self.s
                .tick(c, &self.sys, &mut self.store, now, &mut self.st)
        }
    }

    #[test]
    fn activates_for_enabled_game_and_writes_ahead() {
        let mut r = rig();
        r.sys.run("Game.exe");
        assert!(r.tick(&cfg(&[("game.exe", true)]), 100));
        assert_eq!(r.st.state, GamingState::Active);
        assert_eq!(r.st.game.as_deref(), Some("game.exe"));
        assert_eq!(r.st.sessions, 1);
        assert!(*r.sys.timer_on.borrow());
        assert_eq!(*r.sys.scheme.borrow(), FAKE_ULTIMATE);
        assert_eq!(
            r.store.0.as_ref().unwrap().previous_scheme.as_deref(),
            Some(FAKE_BALANCED)
        );
    }

    #[test]
    fn ignores_disabled_game() {
        let mut r = rig();
        r.sys.run("game.exe");
        assert!(!r.tick(&cfg(&[("game.exe", false)]), 100));
        assert_eq!(r.st.state, GamingState::Idle);
        assert!(!*r.sys.timer_on.borrow());
    }

    #[test]
    fn deactivates_and_restores_previous_plan_when_game_closes() {
        let mut r = rig();
        let c = cfg(&[("game.exe", true)]);
        r.sys.run("game.exe");
        r.tick(&c, 100);
        r.sys.kill("game.exe");
        assert!(r.tick(&c, 160));
        assert_eq!(r.st.state, GamingState::Idle);
        assert_eq!(*r.sys.scheme.borrow(), FAKE_BALANCED);
        assert!(!*r.sys.timer_on.borrow());
        assert!(r.store.0.is_none());
        assert_eq!(r.st.seconds_in_game, 60);
    }

    #[test]
    fn switching_games_starts_a_new_session() {
        let mut r = rig();
        let c = cfg(&[("a.exe", true), ("b.exe", true)]);
        r.sys.run("a.exe");
        r.tick(&c, 100);
        r.sys.kill("a.exe");
        r.sys.run("b.exe");
        r.tick(&c, 130);
        assert_eq!(r.st.game.as_deref(), Some("b.exe"));
        assert_eq!(r.st.sessions, 2);
        assert_eq!(*r.sys.scheme.borrow(), FAKE_ULTIMATE);
        assert_eq!(
            r.store.0.as_ref().unwrap().previous_scheme.as_deref(),
            Some(FAKE_BALANCED)
        );
    }

    #[test]
    fn purges_only_below_limit_and_respecting_interval() {
        let mut r = rig();
        let c = cfg(&[("game.exe", true)]);
        r.sys.run("game.exe");
        *r.sys.free.borrow_mut() = 5000;
        r.tick(&c, 100);
        r.tick(&c, 103);
        assert_eq!(*r.sys.purges.borrow(), 0, "memória sobrando: não limpa");
        *r.sys.free.borrow_mut() = 500;
        r.tick(&c, 106);
        assert_eq!(*r.sys.purges.borrow(), 1);
        assert_eq!(r.st.purged_mb, 3000);
        *r.sys.free.borrow_mut() = 500;
        r.tick(&c, 200);
        assert_eq!(
            *r.sys.purges.borrow(),
            1,
            "dentro dos 300 s: não limpa de novo"
        );
        r.tick(&c, 406);
        assert_eq!(*r.sys.purges.borrow(), 2);
    }

    #[test]
    fn keeps_plan_when_already_ultimate() {
        let mut r = rig();
        *r.sys.scheme.borrow_mut() = FAKE_ULTIMATE.into();
        let c = cfg(&[("game.exe", true)]);
        r.sys.run("game.exe");
        r.tick(&c, 100);
        assert_eq!(r.store.0.as_ref().unwrap().previous_scheme, None);
        r.sys.kill("game.exe");
        r.tick(&c, 110);
        assert_eq!(*r.sys.scheme.borrow(), FAKE_ULTIMATE);
    }

    #[test]
    fn recovers_interrupted_session_on_start() {
        let sys = FakeGamingSys::default();
        *sys.scheme.borrow_mut() = FAKE_ULTIMATE.into();
        let mut store = MemStore(Some(PersistedSession {
            game: "game.exe".into(),
            previous_scheme: Some(FAKE_BALANCED.into()),
            started: 1,
        }));
        Session::recover(&sys, &mut store);
        assert_eq!(*sys.scheme.borrow(), FAKE_BALANCED);
        assert!(store.0.is_none());
    }

    #[test]
    fn disabled_config_deactivates() {
        let mut r = rig();
        let mut c = cfg(&[("game.exe", true)]);
        r.sys.run("game.exe");
        r.tick(&c, 100);
        c.enabled = false;
        assert!(r.tick(&c, 110));
        assert_eq!(r.st.state, GamingState::Idle);
        assert_eq!(*r.sys.scheme.borrow(), FAKE_BALANCED);
    }
}
