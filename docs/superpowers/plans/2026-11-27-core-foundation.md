# Open Optimizer — Subprojeto 1 (Núcleo, UI, Motor de Reversão, Auto-update) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** App Tauri instalável que mostra métricas reais, aplica/desfaz 5 tweaks reais via motor de reversão com journal write-ahead, e se auto-atualiza por GitHub Releases.

**Architecture:** Workspace Cargo com crates puras (`oo-sys` = acesso ao SO atrás do trait `SystemBackend`; `oo-core` = motor + journal, testado com `FakeBackend`; `oo-catalog` = tweaks em TOML + código; `oo-metrics` = leituras Win32). Crate `app` (Tauri 2) expõe comandos IPC. UI em Svelte 5 + Vite com os design tokens do mockup aprovado (`docs/design/mockup-glacier-v3.html`).

**Tech Stack:** Rust 1.97 (edition 2021), Tauri 2, `winreg` 0.55, `windows` 0.61, `serde`/`serde_json`/`toml`, `ulid`, `chrono`, `thiserror`, `ts-rs` 10; Svelte 5, Vite 6, TypeScript 5, Vitest 3, `@fontsource-variable/geist` + `geist-mono`; GitHub Actions + `tauri-apps/tauri-action@v0`.

## Global Constraints

- Plataforma: Windows 10/11 x64. Código Windows atrás de `#[cfg(windows)]`; `oo-core` e `oo-catalog` compilam e testam em qualquer SO.
- O app **nunca** abre handle/injeta/lê/escreve memória de processo de jogo.
- Todo valor mostrado na UI vem de leitura real; nenhum número fixo como dado.
- Instalador ≤ 15 MB; RAM em repouso ≤ 60 MB.
- Journal: `%ProgramData%\OpenOptimizer\journal.jsonl`; logs: `%ProgramData%\OpenOptimizer\logs\`.
- Timestamps RFC 3339 UTC; IDs ULID.
- Cores: `--bg #0B0C0E`, `--panel #0F1013`, `--card #121317`, `--card-h #15171B`, `--line #1C1E23`, `--line-2 #25282E`, `--ice #EEF3F7`, `--ice-2 #C9D3DC`, `--mut #7A828D`, `--dim #4B515A`, `--ok #8FCFB4`. Sem glow, blur ou degradê decorativo.
- Curvas: `--ease: cubic-bezier(.2,.8,.2,1)`, `--spring: cubic-bezier(.34,1.4,.64,1)`. Animar só `transform`/`opacity`. Respeitar `prefers-reduced-motion`.
- Switch: `flex: none` obrigatório.
- i18n: `pt-BR` e `en`; textos de tweaks vivem na UI, chaveados pelo `id` do tweak.
- Endpoint de update: `https://github.com/LucianoBarral/OpenOptimizer/releases/latest/download/latest.json`.
- Comandos que rodam o app (`tauri dev`, `tauri build`) exigem terminal **como administrador** (manifest `requireAdministrator`).
- Commits terminam com `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.

## File Structure

```
Cargo.toml                                  workspace (+ [profile.release])
package.json                                @tauri-apps/cli + script "tauri"
crates/oo-sys/src/lib.rs                    tipos (Hive, RegKey, RegValue), trait SystemBackend, SysError
crates/oo-sys/src/fake.rs                   FakeBackend (HashMap, injeção de falha)
crates/oo-sys/src/windows.rs                WindowsBackend (winreg + powercfg + SPI_SETMOUSE)
crates/oo-sys/src/powercfg.rs               parse de saída do powercfg (puro, testável)
crates/oo-sys/tests/windows_registry.rs     integração real em HKCU\Software\OpenOptimizer\_test
crates/oo-core/src/model.rs                 Action, Before, After, Step, Change, TweakDef, TweakState, OoError
crates/oo-core/src/journal.rs               Journal (JSONL append + fold)
crates/oo-core/src/engine.rs                Engine: apply, revert, detect, history, pending, score
crates/oo-catalog/src/lib.rs + tweaks.toml  catálogo
crates/oo-metrics/src/lib.rs                cpu, memória, timer, hardware
app/…                                       Tauri: build.rs, app.manifest, tauri.conf.json, capabilities, src/{main,commands,views,restore}.rs
ui/src/lib/design/                          tokens.css, Switch, Button, Card, Ring, Sparkline
ui/src/lib/shell/                           TitleBar, Sidebar, UpdateBanner
ui/src/lib/i18n/                            index.ts, pt-BR.ts, en.ts (+ teste de paridade)
ui/src/lib/ipc/                             api.ts + bindings/ (gerado por ts-rs)
ui/src/lib/{format,stores,update}.ts
ui/src/routes/                              Overview, Performance, History, Settings, ComingSoon
.github/workflows/{ci,release}.yml, scripts/bump-version.mjs
```

---

### Task 1: Workspace + `oo-sys` (tipos, trait, FakeBackend)

**Files:**
- Create: `Cargo.toml`, `crates/oo-sys/Cargo.toml`, `crates/oo-sys/src/{lib,fake,powercfg,windows}.rs`

**Interfaces:**
- Produces:
  - `enum Hive { Hklm, Hkcu }`; `struct RegKey { hive, path: String, name: String }` + `RegKey::new(Hive, &str, &str)`
  - `enum RegValue { Dword(u32), Sz(String) }`
  - `trait SystemBackend: Send + Sync` com `reg_get(&RegKey)->Result<Option<RegValue>,SysError>`, `reg_set(&RegKey,&RegValue)`, `reg_delete(&RegKey)`, `power_active()->Result<String,_>`, `power_duplicate(&str)->Result<String,_>`, `power_set_active(&str)`, `power_delete(&str)`, `refresh_mouse()`
  - `enum SysError { AccessDenied, NotFound, Win32(i32), Io(String) }`
  - `FakeBackend::new()`, `.fail_on_set(name)`, `.mouse_refreshes() -> u32`

- [ ] **Step 1: Workspace**

`Cargo.toml`:
```toml
[workspace]
resolver = "2"
members = ["crates/*"]   # "app" entra na Task 8

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "MIT"

[workspace.dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"

[profile.release]
opt-level = "s"
lto = true
codegen-units = 1
strip = true
panic = "abort"
```

`crates/oo-sys/Cargo.toml`:
```toml
[package]
name = "oo-sys"
version.workspace = true
edition.workspace = true

[dependencies]
serde.workspace = true
thiserror.workspace = true

[target.'cfg(windows)'.dependencies]
winreg = "0.55"
windows = { version = "0.61", features = ["Win32_UI_WindowsAndMessaging", "Win32_Foundation"] }

[target.'cfg(windows)'.dev-dependencies]
winreg = "0.55"
```

- [ ] **Step 2: Testes falhando** — fim de `crates/oo-sys/src/fake.rs`:

```rust
#[cfg(test)]
mod tests {
    use crate::*;

    fn k() -> RegKey { RegKey::new(Hive::Hkcu, r"Software\T", "V") }

    #[test]
    fn get_missing_is_none() { assert_eq!(FakeBackend::new().reg_get(&k()).unwrap(), None); }

    #[test]
    fn set_get_delete_roundtrip() {
        let b = FakeBackend::new();
        b.reg_set(&k(), &RegValue::Dword(7)).unwrap();
        assert_eq!(b.reg_get(&k()).unwrap(), Some(RegValue::Dword(7)));
        b.reg_delete(&k()).unwrap();
        assert_eq!(b.reg_get(&k()).unwrap(), None);
    }

    #[test]
    fn injected_failure() {
        let b = FakeBackend::new();
        b.fail_on_set("V");
        assert!(matches!(b.reg_set(&k(), &RegValue::Dword(1)), Err(SysError::AccessDenied)));
    }

    #[test]
    fn power_duplicate_and_delete() {
        let b = FakeBackend::new();
        let orig = b.power_active().unwrap();
        let g = b.power_duplicate("e9a42b02-d5df-448d-aa00-03f14749eb61").unwrap();
        b.power_set_active(&g).unwrap();
        assert_eq!(b.power_active().unwrap(), g);
        assert!(b.power_delete(&g).is_err(), "não pode apagar o plano ativo");
        b.power_set_active(&orig).unwrap();
        b.power_delete(&g).unwrap();
        assert!(b.power_set_active(&g).is_err());
    }
}
```

- [ ] **Step 3: Rodar e ver falhar** — `cargo test -p oo-sys` — Expected: FAIL (não compila).

- [ ] **Step 4: `lib.rs`**

```rust
use serde::{Deserialize, Serialize};

pub mod fake;
pub mod powercfg;
#[cfg(windows)]
pub mod windows;

pub use fake::FakeBackend;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Hive { Hklm, Hkcu }

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RegKey { pub hive: Hive, pub path: String, pub name: String }

impl RegKey {
    pub fn new(hive: Hive, path: &str, name: &str) -> Self {
        Self { hive, path: path.to_string(), name: name.to_string() }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegValue { Dword(u32), Sz(String) }

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, Serialize, Deserialize)]
pub enum SysError {
    #[error("access denied")] AccessDenied,
    #[error("not found")] NotFound,
    #[error("win32 error {0}")] Win32(i32),
    #[error("io: {0}")] Io(String),
}

pub trait SystemBackend: Send + Sync {
    fn reg_get(&self, key: &RegKey) -> Result<Option<RegValue>, SysError>;
    fn reg_set(&self, key: &RegKey, value: &RegValue) -> Result<(), SysError>;
    fn reg_delete(&self, key: &RegKey) -> Result<(), SysError>;
    fn power_active(&self) -> Result<String, SysError>;
    fn power_duplicate(&self, source_guid: &str) -> Result<String, SysError>;
    fn power_set_active(&self, guid: &str) -> Result<(), SysError>;
    fn power_delete(&self, guid: &str) -> Result<(), SysError>;
    fn refresh_mouse(&self) -> Result<(), SysError>;
}
```
`powercfg.rs` e `windows.rs`: apenas `//! Implementado na Task 5.` por enquanto.

- [ ] **Step 5: `fake.rs`** (acima dos testes):

```rust
use crate::*;
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

#[derive(Default)]
struct State {
    reg: HashMap<RegKey, RegValue>,
    fail_set: HashSet<String>,
    schemes: Vec<String>,
    active: String,
    next: u32,
    mouse_refreshes: u32,
}

pub struct FakeBackend(Mutex<State>);

impl FakeBackend {
    pub fn new() -> Self {
        let balanced = "381b4222-f694-41f0-9685-ff5bb260df2e".to_string();
        Self(Mutex::new(State { schemes: vec![balanced.clone()], active: balanced, ..Default::default() }))
    }
    pub fn fail_on_set(&self, name: &str) { self.0.lock().unwrap().fail_set.insert(name.to_string()); }
    pub fn mouse_refreshes(&self) -> u32 { self.0.lock().unwrap().mouse_refreshes }
}

impl Default for FakeBackend { fn default() -> Self { Self::new() } }

impl SystemBackend for FakeBackend {
    fn reg_get(&self, key: &RegKey) -> Result<Option<RegValue>, SysError> {
        Ok(self.0.lock().unwrap().reg.get(key).cloned())
    }
    fn reg_set(&self, key: &RegKey, value: &RegValue) -> Result<(), SysError> {
        let mut s = self.0.lock().unwrap();
        if s.fail_set.contains(&key.name) { return Err(SysError::AccessDenied); }
        s.reg.insert(key.clone(), value.clone());
        Ok(())
    }
    fn reg_delete(&self, key: &RegKey) -> Result<(), SysError> {
        self.0.lock().unwrap().reg.remove(key);
        Ok(())
    }
    fn power_active(&self) -> Result<String, SysError> { Ok(self.0.lock().unwrap().active.clone()) }
    fn power_duplicate(&self, _source: &str) -> Result<String, SysError> {
        let mut s = self.0.lock().unwrap();
        s.next += 1;
        let g = format!("00000000-0000-0000-0000-{:012}", s.next);
        s.schemes.push(g.clone());
        Ok(g)
    }
    fn power_set_active(&self, guid: &str) -> Result<(), SysError> {
        let mut s = self.0.lock().unwrap();
        if !s.schemes.iter().any(|g| g == guid) { return Err(SysError::NotFound); }
        s.active = guid.to_string();
        Ok(())
    }
    fn power_delete(&self, guid: &str) -> Result<(), SysError> {
        let mut s = self.0.lock().unwrap();
        if s.active == guid { return Err(SysError::AccessDenied); }
        s.schemes.retain(|g| g != guid);
        Ok(())
    }
    fn refresh_mouse(&self) -> Result<(), SysError> { self.0.lock().unwrap().mouse_refreshes += 1; Ok(()) }
}
```

- [ ] **Step 6: Rodar** — `cargo test -p oo-sys` — Expected: 4 passed.
- [ ] **Step 7: Commit** — `git add Cargo.toml crates/oo-sys && git commit -m "feat(oo-sys): trait SystemBackend e FakeBackend"`

---

### Task 2: `oo-core` modelo + Journal

**Files:**
- Create: `crates/oo-core/Cargo.toml`, `crates/oo-core/src/{lib,model,journal,engine}.rs`

**Interfaces:**
- Consumes: `oo_sys::{RegKey, RegValue, SysError}`
- Produces:
  - `enum Action { RegSet { key, value }, RegDelete { key }, PowerPlan { source_guid: String } }`
  - `enum Before { Reg(Option<RegValue>), Power { previous: String } }`; `enum After { Reg(Option<RegValue>), Power { created: Option<String> } }`
  - `struct Step { action, before, after }`; `enum ChangeStatus { Pending, Applied, Reverted, Failed }`
  - `struct Change { id, tweak_id, batch_id, timestamp: String, steps: Vec<Step>, status: ChangeStatus, post: Option<PostHook> }`
  - `enum PostHook { RefreshMouse }`; `enum Risk { Safe, Advanced }`
  - `struct TweakDef { id, category: String, risk: Risk, requires_reboot: bool, weight: u32, actions: Vec<Action>, post: Option<PostHook> }`
  - `enum TweakState { Applied, NotApplied, Partial, Unsupported }`
  - `enum OoError { Sys(SysError), Conflict { change_id }, NotFound(String), Io(String) }`
  - `Journal::open(PathBuf) -> Result<Journal, OoError>`; `record(&mut self, &Change) -> Result<(), OoError>`; `changes(&self) -> Vec<Change>` (último estado por id, ordem de criação)

- [ ] **Step 1: `Cargo.toml`**

```toml
[package]
name = "oo-core"
version.workspace = true
edition.workspace = true

[dependencies]
oo-sys = { path = "../oo-sys" }
serde.workspace = true
serde_json.workspace = true
thiserror.workspace = true
ulid = "1"
chrono = { version = "0.4", default-features = false, features = ["clock", "std"] }

[dev-dependencies]
tempfile = "3"
```

- [ ] **Step 2: Testes falhando** — fim de `journal.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::*;
    use std::io::Write;

    fn change(id: &str, status: ChangeStatus) -> Change {
        Change { id: id.into(), tweak_id: "t".into(), batch_id: "b".into(),
                 timestamp: "2026-11-27T10:00:00Z".into(), steps: vec![], status, post: None }
    }

    #[test]
    fn later_record_wins_and_order_is_stable() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("j.jsonl");
        let mut j = Journal::open(p.clone()).unwrap();
        j.record(&change("a", ChangeStatus::Pending)).unwrap();
        j.record(&change("b", ChangeStatus::Pending)).unwrap();
        j.record(&change("a", ChangeStatus::Applied)).unwrap();
        let c = Journal::open(p).unwrap().changes();
        assert_eq!(c.iter().map(|c| c.id.as_str()).collect::<Vec<_>>(), vec!["a", "b"]);
        assert_eq!(c[0].status, ChangeStatus::Applied);
    }

    #[test]
    fn truncated_last_line_is_ignored() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("j.jsonl");
        Journal::open(p.clone()).unwrap().record(&change("a", ChangeStatus::Applied)).unwrap();
        std::fs::OpenOptions::new().append(true).open(&p).unwrap().write_all(b"{\"id\":\"tru").unwrap();
        assert_eq!(Journal::open(p).unwrap().changes().len(), 1);
    }
}
```
`lib.rs`:
```rust
pub mod engine;
pub mod journal;
pub mod model;
pub use engine::Engine;
pub use journal::Journal;
pub use model::*;
```
`engine.rs` por enquanto: `pub struct Engine;`

- [ ] **Step 3: Rodar e ver falhar** — `cargo test -p oo-core` — FAIL.

- [ ] **Step 4: `model.rs`**

```rust
use oo_sys::{RegKey, RegValue, SysError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    RegSet { key: RegKey, value: RegValue },
    RegDelete { key: RegKey },
    PowerPlan { source_guid: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Before { Reg(Option<RegValue>), Power { previous: String } }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum After { Reg(Option<RegValue>), Power { created: Option<String> } }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Step { pub action: Action, pub before: Before, pub after: After }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeStatus { Pending, Applied, Reverted, Failed }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PostHook { RefreshMouse }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Change {
    pub id: String, pub tweak_id: String, pub batch_id: String, pub timestamp: String,
    pub steps: Vec<Step>, pub status: ChangeStatus,
    #[serde(default)] pub post: Option<PostHook>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Risk { Safe, Advanced }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TweakDef {
    pub id: String, pub category: String, pub risk: Risk, pub requires_reboot: bool,
    pub weight: u32, pub actions: Vec<Action>, pub post: Option<PostHook>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TweakState { Applied, NotApplied, Partial, Unsupported }

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, Serialize, Deserialize)]
pub enum OoError {
    #[error("system: {0}")] Sys(#[from] SysError),
    #[error("conflict in change {change_id}")] Conflict { change_id: String },
    #[error("not found: {0}")] NotFound(String),
    #[error("io: {0}")] Io(String),
}

impl From<std::io::Error> for OoError {
    fn from(e: std::io::Error) -> Self { OoError::Io(e.to_string()) }
}
```

- [ ] **Step 5: `journal.rs`** (acima dos testes):

```rust
use crate::model::{Change, OoError};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

pub struct Journal { path: PathBuf, order: Vec<String>, latest: HashMap<String, Change> }

impl Journal {
    pub fn open(path: PathBuf) -> Result<Self, OoError> {
        if let Some(dir) = path.parent() { std::fs::create_dir_all(dir)?; }
        let mut j = Journal { path, order: vec![], latest: HashMap::new() };
        if let Ok(f) = File::open(&j.path) {
            for line in BufReader::new(f).lines() {
                let Ok(line) = line else { break };
                // Linha truncada (crash no meio da escrita) não desserializa e é descartada.
                if let Ok(c) = serde_json::from_str::<Change>(&line) { j.fold(c); }
            }
        }
        Ok(j)
    }

    fn fold(&mut self, c: Change) {
        if !self.latest.contains_key(&c.id) { self.order.push(c.id.clone()); }
        self.latest.insert(c.id.clone(), c);
    }

    /// Write-ahead: só retorna depois de o dado estar no disco (sync_all = FlushFileBuffers).
    pub fn record(&mut self, c: &Change) -> Result<(), OoError> {
        let mut f = OpenOptions::new().create(true).append(true).open(&self.path)?;
        let mut line = serde_json::to_string(c).map_err(|e| OoError::Io(e.to_string()))?;
        line.push('\n');
        f.write_all(line.as_bytes())?;
        f.sync_all()?;
        self.fold(c.clone());
        Ok(())
    }

    pub fn changes(&self) -> Vec<Change> { self.order.iter().map(|id| self.latest[id].clone()).collect() }
}
```

- [ ] **Step 6: Rodar** — `cargo test -p oo-core` — Expected: 2 passed.
- [ ] **Step 7: Commit** — `git add crates/oo-core && git commit -m "feat(oo-core): modelo e journal write-ahead"`

---

### Task 3: `oo-core` Engine

**Files:**
- Modify: `crates/oo-core/src/engine.rs` (substituir conteúdo)

**Interfaces:**
- Consumes: Tasks 1–2
- Produces:
  - `Engine<B: SystemBackend>::new(B, Journal)`, `backend(&self) -> &B`
  - `apply(&mut self, &TweakDef, batch_id: &str) -> Result<Change, OoError>`
  - `revert(&mut self, change_id: &str, force: bool) -> Result<(), OoError>`
  - `revert_tweak(&mut self, tweak_id: &str, force: bool) -> Result<(), OoError>`
  - `revert_all(&mut self, force: bool) -> Vec<(String, OoError)>`
  - `detect(&self, &TweakDef) -> TweakState`; `history()`, `pending() -> Vec<Change>`; `score(&self, &[TweakDef]) -> u8`
  - função livre `new_batch_id() -> String`

- [ ] **Step 1: Testes falhando** — fim de `engine.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use oo_sys::{FakeBackend, Hive, RegKey, RegValue, SystemBackend};

    fn key(n: &str) -> RegKey { RegKey::new(Hive::Hkcu, r"Software\T", n) }
    fn set(n: &str, v: u32) -> Action { Action::RegSet { key: key(n), value: RegValue::Dword(v) } }
    fn tweak(id: &str, actions: Vec<Action>) -> TweakDef {
        TweakDef { id: id.into(), category: "fps".into(), risk: Risk::Safe, requires_reboot: false,
                   weight: 1, actions, post: None }
    }
    fn eng() -> (Engine<FakeBackend>, tempfile::TempDir) {
        let d = tempfile::tempdir().unwrap();
        let j = Journal::open(d.path().join("j.jsonl")).unwrap();
        (Engine::new(FakeBackend::new(), j), d)
    }

    #[test]
    fn apply_then_revert_restores_previous_value() {
        let (mut e, _d) = eng();
        e.backend().reg_set(&key("A"), &RegValue::Dword(20)).unwrap();
        let t = tweak("t", vec![set("A", 10)]);
        let c = e.apply(&t, "b1").unwrap();
        assert_eq!(e.backend().reg_get(&key("A")).unwrap(), Some(RegValue::Dword(10)));
        assert_eq!(e.detect(&t), TweakState::Applied);
        e.revert(&c.id, false).unwrap();
        assert_eq!(e.backend().reg_get(&key("A")).unwrap(), Some(RegValue::Dword(20)));
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
        assert!(e.apply(&tweak("t", vec![set("A", 1), set("B", 1)]), "b").is_err());
        assert_eq!(e.backend().reg_get(&key("A")).unwrap(), None);
        assert_eq!(e.history()[0].status, ChangeStatus::Failed);
    }

    #[test]
    fn external_change_is_a_conflict_unless_forced() {
        let (mut e, _d) = eng();
        let c = e.apply(&tweak("t", vec![set("A", 1)]), "b").unwrap();
        e.backend().reg_set(&key("A"), &RegValue::Dword(99)).unwrap();
        assert!(matches!(e.revert(&c.id, false), Err(OoError::Conflict { .. })));
        assert_eq!(e.backend().reg_get(&key("A")).unwrap(), Some(RegValue::Dword(99)));
        e.revert(&c.id, true).unwrap();
        assert_eq!(e.backend().reg_get(&key("A")).unwrap(), None);
    }

    #[test]
    fn power_plan_apply_and_revert() {
        let (mut e, _d) = eng();
        let orig = e.backend().power_active().unwrap();
        let t = tweak("pp", vec![Action::PowerPlan { source_guid: "e9a42b02-d5df-448d-aa00-03f14749eb61".into() }]);
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
        Journal::open(p.clone()).unwrap().record(&Change {
            id: "c1".into(), tweak_id: "t".into(), batch_id: "b".into(),
            timestamp: "2026-11-27T10:00:00Z".into(), status: ChangeStatus::Pending, post: None,
            steps: vec![Step { action: set("A", 1),
                before: Before::Reg(Some(RegValue::Dword(5))), after: After::Reg(Some(RegValue::Dword(1))) }],
        }).unwrap();
        b.reg_set(&key("A"), &RegValue::Dword(1)).unwrap();
        let mut e = Engine::new(b, Journal::open(p).unwrap());
        assert_eq!(e.pending().len(), 1);
        e.revert("c1", true).unwrap();
        assert_eq!(e.backend().reg_get(&key("A")).unwrap(), Some(RegValue::Dword(5)));
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
        let mut t = tweak("m", vec![Action::RegSet { key: key("M"), value: RegValue::Sz("0".into()) }]);
        t.post = Some(PostHook::RefreshMouse);
        let c = e.apply(&t, "b").unwrap();
        e.revert(&c.id, false).unwrap();
        assert_eq!(e.backend().mouse_refreshes(), 2);
    }
}
```

- [ ] **Step 2: Rodar e ver falhar** — `cargo test -p oo-core engine` — FAIL.

- [ ] **Step 3: Implementar** (acima dos testes):

```rust
use crate::journal::Journal;
use crate::model::*;
use oo_sys::SystemBackend;

pub fn new_batch_id() -> String { ulid::Ulid::new().to_string() }
fn now() -> String { chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true) }

pub struct Engine<B: SystemBackend> { backend: B, journal: Journal }

impl<B: SystemBackend> Engine<B> {
    pub fn new(backend: B, journal: Journal) -> Self { Self { backend, journal } }
    pub fn backend(&self) -> &B { &self.backend }
    pub fn history(&self) -> Vec<Change> { self.journal.changes() }
    pub fn pending(&self) -> Vec<Change> {
        self.history().into_iter().filter(|c| c.status == ChangeStatus::Pending).collect()
    }

    fn capture(&self, a: &Action) -> Result<Step, OoError> {
        Ok(match a {
            Action::RegSet { key, value } => Step { action: a.clone(),
                before: Before::Reg(self.backend.reg_get(key)?), after: After::Reg(Some(value.clone())) },
            Action::RegDelete { key } => Step { action: a.clone(),
                before: Before::Reg(self.backend.reg_get(key)?), after: After::Reg(None) },
            Action::PowerPlan { .. } => Step { action: a.clone(),
                before: Before::Power { previous: self.backend.power_active()? },
                after: After::Power { created: None } },
        })
    }

    fn execute(&self, step: &mut Step) -> Result<(), OoError> {
        match &step.action {
            Action::RegSet { key, value } => self.backend.reg_set(key, value)?,
            Action::RegDelete { key } => self.backend.reg_delete(key)?,
            Action::PowerPlan { source_guid } => {
                let g = self.backend.power_duplicate(source_guid)?;
                step.after = After::Power { created: Some(g.clone()) };
                self.backend.power_set_active(&g)?;
            }
        }
        Ok(())
    }

    fn undo(&self, step: &Step) -> Result<(), OoError> {
        match (&step.action, &step.before, &step.after) {
            (Action::RegSet { key, .. } | Action::RegDelete { key }, Before::Reg(prev), _) => match prev {
                Some(v) => self.backend.reg_set(key, v)?,
                None => self.backend.reg_delete(key)?,
            },
            (Action::PowerPlan { .. }, Before::Power { previous }, After::Power { created }) => {
                self.backend.power_set_active(previous)?;
                if let Some(g) = created { self.backend.power_delete(g)?; }
            }
            _ => return Err(OoError::Io("step inconsistente no journal".into())),
        }
        Ok(())
    }

    fn run_post(&self, post: Option<PostHook>) -> Result<(), OoError> {
        if let Some(PostHook::RefreshMouse) = post { self.backend.refresh_mouse()?; }
        Ok(())
    }

    pub fn apply(&mut self, t: &TweakDef, batch_id: &str) -> Result<Change, OoError> {
        let steps = t.actions.iter().map(|a| self.capture(a)).collect::<Result<Vec<_>, _>>()?;
        let mut c = Change { id: ulid::Ulid::new().to_string(), tweak_id: t.id.clone(), batch_id: batch_id.into(),
            timestamp: now(), steps, status: ChangeStatus::Pending, post: t.post };
        self.journal.record(&c)?; // write-ahead: originais no disco antes de qualquer alteração
        for i in 0..c.steps.len() {
            let mut step = c.steps[i].clone();
            let res = self.execute(&mut step);
            c.steps[i] = step;
            if let Err(e) = res {
                for done in c.steps[..i].iter().rev() { let _ = self.undo(done); }
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

    fn step_is_intact(&self, s: &Step) -> Result<bool, OoError> {
        Ok(match (&s.action, &s.after) {
            (Action::RegSet { key, .. } | Action::RegDelete { key }, After::Reg(expected)) =>
                &self.backend.reg_get(key)? == expected,
            (Action::PowerPlan { .. }, After::Power { created }) =>
                created.as_deref() == Some(self.backend.power_active()?.as_str()),
            _ => false,
        })
    }

    pub fn revert(&mut self, change_id: &str, force: bool) -> Result<(), OoError> {
        let mut c = self.history().into_iter().find(|c| c.id == change_id)
            .ok_or_else(|| OoError::NotFound(change_id.into()))?;
        if !force {
            for s in &c.steps {
                if !self.step_is_intact(s)? { return Err(OoError::Conflict { change_id: c.id.clone() }); }
            }
        }
        for s in c.steps.iter().rev() { self.undo(s)?; }
        self.run_post(c.post)?;
        c.status = ChangeStatus::Reverted;
        c.timestamp = now();
        self.journal.record(&c)
    }

    fn active_change_for(&self, tweak_id: &str) -> Option<Change> {
        self.history().into_iter().rev().find(|c| c.tweak_id == tweak_id && c.status == ChangeStatus::Applied)
    }

    pub fn revert_tweak(&mut self, tweak_id: &str, force: bool) -> Result<(), OoError> {
        let c = self.active_change_for(tweak_id).ok_or_else(|| OoError::NotFound(tweak_id.into()))?;
        self.revert(&c.id, force)
    }

    pub fn revert_all(&mut self, force: bool) -> Vec<(String, OoError)> {
        let ids: Vec<String> = self.history().into_iter().rev()
            .filter(|c| matches!(c.status, ChangeStatus::Applied | ChangeStatus::Pending))
            .map(|c| c.id).collect();
        ids.into_iter().filter_map(|id| self.revert(&id, force).err().map(|e| (id, e))).collect()
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
                Action::PowerPlan { .. } => self.active_change_for(&t.id)
                    .is_some_and(|c| c.steps.iter().any(|s| self.step_is_intact(s).unwrap_or(false))),
            };
            if ok { hit += 1; }
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
                TweakState::Applied => { got += t.weight; total += t.weight; }
                _ => total += t.weight,
            }
        }
        if total == 0 { 100 } else { (got * 100 / total) as u8 }
    }
}
```

- [ ] **Step 4: Rodar** — `cargo test -p oo-core` — Expected: 10 passed.
- [ ] **Step 5: Clippy** — `cargo clippy -p oo-core -p oo-sys -- -D warnings` — Expected: limpo.
- [ ] **Step 6: Commit** — `git commit -am "feat(oo-core): engine com apply/revert, conflito, recuperação e pontuação"`

---

### Task 4: `oo-catalog`

**Files:**
- Create: `crates/oo-catalog/Cargo.toml`, `crates/oo-catalog/src/lib.rs`, `crates/oo-catalog/tweaks.toml`

**Interfaces:**
- Produces: `pub fn load_catalog() -> Vec<TweakDef>` com ids, nesta ordem: `power-throttling-off`, `mmcss-games`, `game-dvr-off`, `mouse-accel-off`, `ultimate-power-plan`.

- [ ] **Step 1: `Cargo.toml`**

```toml
[package]
name = "oo-catalog"
version.workspace = true
edition.workspace = true

[dependencies]
oo-core = { path = "../oo-core" }
oo-sys = { path = "../oo-sys" }
serde.workspace = true
toml = "0.8"
```

- [ ] **Step 2: Teste falhando** — fim de `lib.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn has_the_five_starter_tweaks() {
        let c = load_catalog();
        let ids: Vec<_> = c.iter().map(|t| t.id.as_str()).collect();
        assert_eq!(ids, ["power-throttling-off", "mmcss-games", "game-dvr-off", "mouse-accel-off", "ultimate-power-plan"]);
        assert!(c.iter().all(|t| !t.actions.is_empty() && t.weight > 0));
        assert_eq!(c.iter().find(|t| t.id == "mouse-accel-off").unwrap().post, Some(PostHook::RefreshMouse));
    }
}
```
Run: `cargo test -p oo-catalog` — FAIL.

- [ ] **Step 3: `tweaks.toml`**

```toml
# `rationale` é obrigatório: justificativa técnica (exibida na UI via chave i18n tweak.<id>.why).

[[tweak]]
id = "power-throttling-off"
category = "energy"
weight = 2
rationale = "Power Throttling reduz a frequência de processos que o Windows considera em segundo plano; jogos sem foco e launchers sofrem quedas."
set = [
  { hive = "HKLM", path = 'SYSTEM\CurrentControlSet\Control\Power\PowerThrottling', name = "PowerThrottlingOff", dword = 1 },
]

[[tweak]]
id = "mmcss-games"
category = "fps"
weight = 3
rationale = "MMCSS reserva CPU para multimídia; SystemResponsiveness=10 deixa só 10% para segundo plano e a tarefa Games ganha prioridade alta de CPU e GPU."
set = [
  { hive = "HKLM", path = 'SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile', name = "SystemResponsiveness", dword = 10 },
  { hive = "HKLM", path = 'SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile\Tasks\Games', name = "GPU Priority", dword = 8 },
  { hive = "HKLM", path = 'SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile\Tasks\Games', name = "Priority", dword = 6 },
  { hive = "HKLM", path = 'SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile\Tasks\Games', name = "Scheduling Category", sz = "High" },
]

[[tweak]]
id = "game-dvr-off"
category = "fps"
weight = 2
rationale = "A gravação em segundo plano do Xbox Game Bar mantém um encoder de vídeo ativo durante o jogo, consumindo GPU e VRAM."
set = [
  { hive = "HKCU", path = 'System\GameConfigStore', name = "GameDVR_Enabled", dword = 0 },
  { hive = "HKCU", path = 'Software\Microsoft\Windows\CurrentVersion\GameDVR', name = "AppCaptureEnabled", dword = 0 },
]

[[tweak]]
id = "mouse-accel-off"
category = "input"
weight = 2
post = "refresh_mouse"
rationale = "'Aprimorar precisão do ponteiro' aplica curva de aceleração: o mesmo movimento físico gera distâncias diferentes conforme a velocidade."
set = [
  { hive = "HKCU", path = 'Control Panel\Mouse', name = "MouseSpeed", sz = "0" },
  { hive = "HKCU", path = 'Control Panel\Mouse', name = "MouseThreshold1", sz = "0" },
  { hive = "HKCU", path = 'Control Panel\Mouse', name = "MouseThreshold2", sz = "0" },
]
```

- [ ] **Step 4: `lib.rs`** (acima dos testes):

```rust
use oo_core::{Action, PostHook, Risk, TweakDef};
use oo_sys::{Hive, RegKey, RegValue};
use serde::Deserialize;

#[derive(Deserialize)]
struct File { tweak: Vec<Raw> }

#[derive(Deserialize)]
struct Raw {
    id: String, category: String, weight: u32,
    #[serde(default)] requires_reboot: bool,
    #[serde(default)] advanced: bool,
    #[serde(default)] post: Option<String>,
    #[allow(dead_code)] rationale: String,
    set: Vec<RawSet>,
}

#[derive(Deserialize)]
struct RawSet { hive: String, path: String, name: String, dword: Option<u32>, sz: Option<String> }

const ULTIMATE_PERFORMANCE: &str = "e9a42b02-d5df-448d-aa00-03f14749eb61";

pub fn load_catalog() -> Vec<TweakDef> {
    let f: File = toml::from_str(include_str!("../tweaks.toml")).expect("tweaks.toml inválido");
    let mut out: Vec<TweakDef> = f.tweak.into_iter().map(|r| TweakDef {
        id: r.id, category: r.category, weight: r.weight, requires_reboot: r.requires_reboot,
        risk: if r.advanced { Risk::Advanced } else { Risk::Safe },
        post: match r.post.as_deref() {
            None => None,
            Some("refresh_mouse") => Some(PostHook::RefreshMouse),
            Some(o) => panic!("post desconhecido: {o}"),
        },
        actions: r.set.into_iter().map(|s| {
            let hive = match s.hive.as_str() { "HKLM" => Hive::Hklm, "HKCU" => Hive::Hkcu, h => panic!("hive: {h}") };
            let value = match (s.dword, s.sz) {
                (Some(d), None) => RegValue::Dword(d),
                (None, Some(z)) => RegValue::Sz(z),
                _ => panic!("{}: informe exatamente um de dword/sz", s.name),
            };
            Action::RegSet { key: RegKey::new(hive, &s.path, &s.name), value }
        }).collect(),
    }).collect();
    out.push(TweakDef { id: "ultimate-power-plan".into(), category: "energy".into(), weight: 3,
        requires_reboot: false, risk: Risk::Safe, post: None,
        actions: vec![Action::PowerPlan { source_guid: ULTIMATE_PERFORMANCE.into() }] });
    out
}
```
(Os `panic!` só disparam com TOML embutido inválido, e o teste do Step 2 os pega no CI, nunca em runtime.)

- [ ] **Step 5: Rodar** — `cargo test -p oo-catalog` — PASS.
- [ ] **Step 6: Commit** — `git add crates/oo-catalog && git commit -m "feat(oo-catalog): 5 tweaks iniciais"`

---

### Task 5: `WindowsBackend` real + parse do powercfg

**Files:**
- Modify: `crates/oo-sys/src/powercfg.rs`, `crates/oo-sys/src/windows.rs`
- Create: `crates/oo-sys/tests/windows_registry.rs`

**Interfaces:**
- Produces: `oo_sys::windows::WindowsBackend::new()`; `oo_sys::powercfg::parse_guid(&str) -> Option<String>`

- [ ] **Step 1: Parser + teste** — `powercfg.rs`:

```rust
/// Extrai o primeiro GUID (minúsculo) de saídas como
/// "Power Scheme GUID: 381b4222-f694-41f0-9685-ff5bb260df2e  (Balanced)".
/// O texto muda com o idioma do Windows; o GUID não.
pub fn parse_guid(output: &str) -> Option<String> {
    output.split(|c: char| !(c.is_ascii_hexdigit() || c == '-'))
        .find(|t| t.len() == 36 && t.matches('-').count() == 4)
        .map(|s| s.to_ascii_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_english_and_portuguese() {
        assert_eq!(parse_guid("Power Scheme GUID: 381b4222-f694-41f0-9685-ff5bb260df2e  (Balanced)").as_deref(),
                   Some("381b4222-f694-41f0-9685-ff5bb260df2e"));
        assert_eq!(parse_guid("GUID do Esquema de Energia: E9A42B02-D5DF-448D-AA00-03F14749EB61  (Desempenho Máximo)").as_deref(),
                   Some("e9a42b02-d5df-448d-aa00-03f14749eb61"));
        assert_eq!(parse_guid("nada aqui"), None);
    }
}
```
Run: `cargo test -p oo-sys powercfg` — PASS.

- [ ] **Step 2: Integração falhando** — `tests/windows_registry.rs`:

```rust
#![cfg(windows)]
use oo_sys::{windows::WindowsBackend, Hive, RegKey, RegValue, SystemBackend};

#[test]
fn hkcu_set_get_delete() {
    let b = WindowsBackend::new();
    let k = RegKey::new(Hive::Hkcu, r"Software\OpenOptimizer\_test", "Probe");
    b.reg_set(&k, &RegValue::Dword(42)).unwrap();
    assert_eq!(b.reg_get(&k).unwrap(), Some(RegValue::Dword(42)));
    b.reg_set(&k, &RegValue::Sz("x".into())).unwrap();
    assert_eq!(b.reg_get(&k).unwrap(), Some(RegValue::Sz("x".into())));
    b.reg_delete(&k).unwrap();
    assert_eq!(b.reg_get(&k).unwrap(), None);
    b.reg_delete(&k).unwrap(); // idempotente
    let _ = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER).delete_subkey_all(r"Software\OpenOptimizer\_test");
}

#[test]
fn power_active_returns_guid() {
    assert_eq!(WindowsBackend::new().power_active().unwrap().len(), 36);
}
```
Run: `cargo test -p oo-sys --test windows_registry` — FAIL.

- [ ] **Step 3: `windows.rs`**

```rust
use crate::*;
use std::os::windows::process::CommandExt;
use std::process::Command;
use winreg::enums::*;

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

pub struct WindowsBackend;
impl WindowsBackend { pub fn new() -> Self { WindowsBackend } }
impl Default for WindowsBackend { fn default() -> Self { Self::new() } }

fn root(h: Hive) -> winreg::RegKey {
    winreg::RegKey::predef(match h { Hive::Hklm => HKEY_LOCAL_MACHINE, Hive::Hkcu => HKEY_CURRENT_USER })
}
fn is_nf(e: &std::io::Error) -> bool { e.kind() == std::io::ErrorKind::NotFound }
fn map(e: std::io::Error) -> SysError {
    match e.kind() {
        std::io::ErrorKind::NotFound => SysError::NotFound,
        std::io::ErrorKind::PermissionDenied => SysError::AccessDenied,
        _ => e.raw_os_error().map(SysError::Win32).unwrap_or_else(|| SysError::Io(e.to_string())),
    }
}
fn powercfg(args: &[&str]) -> Result<String, SysError> {
    let o = Command::new("powercfg").args(args).creation_flags(CREATE_NO_WINDOW).output()
        .map_err(|e| SysError::Io(e.to_string()))?;
    if !o.status.success() { return Err(SysError::Win32(o.status.code().unwrap_or(-1))); }
    Ok(String::from_utf8_lossy(&o.stdout).into_owned())
}

impl SystemBackend for WindowsBackend {
    fn reg_get(&self, k: &RegKey) -> Result<Option<RegValue>, SysError> {
        let key = match root(k.hive).open_subkey_with_flags(&k.path, KEY_READ) {
            Ok(x) => x, Err(e) if is_nf(&e) => return Ok(None), Err(e) => return Err(map(e)) };
        let raw = match key.get_raw_value(&k.name) {
            Ok(r) => r, Err(e) if is_nf(&e) => return Ok(None), Err(e) => return Err(map(e)) };
        Ok(Some(match raw.vtype {
            REG_DWORD if raw.bytes.len() >= 4 =>
                RegValue::Dword(u32::from_le_bytes([raw.bytes[0], raw.bytes[1], raw.bytes[2], raw.bytes[3]])),
            REG_SZ | REG_EXPAND_SZ => {
                let wide: Vec<u16> = raw.bytes.chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect();
                RegValue::Sz(String::from_utf16_lossy(&wide).trim_end_matches('\0').to_string())
            }
            _ => return Err(SysError::Io(format!("tipo de registro não suportado em {}", k.name))),
        }))
    }
    fn reg_set(&self, k: &RegKey, v: &RegValue) -> Result<(), SysError> {
        let (key, _) = root(k.hive).create_subkey(&k.path).map_err(map)?;
        match v { RegValue::Dword(d) => key.set_value(&k.name, d), RegValue::Sz(s) => key.set_value(&k.name, s) }.map_err(map)
    }
    fn reg_delete(&self, k: &RegKey) -> Result<(), SysError> {
        match root(k.hive).open_subkey_with_flags(&k.path, KEY_SET_VALUE) {
            Ok(key) => match key.delete_value(&k.name) { Ok(()) => Ok(()), Err(e) if is_nf(&e) => Ok(()), Err(e) => Err(map(e)) },
            Err(e) if is_nf(&e) => Ok(()),
            Err(e) => Err(map(e)),
        }
    }
    fn power_active(&self) -> Result<String, SysError> {
        powercfg::parse_guid(&powercfg(&["/getactivescheme"])?).ok_or(SysError::NotFound)
    }
    fn power_duplicate(&self, src: &str) -> Result<String, SysError> {
        powercfg::parse_guid(&powercfg(&["/duplicatescheme", src])?).ok_or(SysError::NotFound)
    }
    fn power_set_active(&self, g: &str) -> Result<(), SysError> { powercfg(&["/setactive", g]).map(|_| ()) }
    fn power_delete(&self, g: &str) -> Result<(), SysError> { powercfg(&["/delete", g]).map(|_| ()) }
    fn refresh_mouse(&self) -> Result<(), SysError> {
        use windows::Win32::UI::WindowsAndMessaging::{SystemParametersInfoW, SPI_SETMOUSE, SPIF_SENDCHANGE, SPIF_UPDATEINIFILE};
        let read = |n: &str| -> i32 {
            match self.reg_get(&RegKey::new(Hive::Hkcu, r"Control Panel\Mouse", n)) {
                Ok(Some(RegValue::Sz(s))) => s.trim().parse().unwrap_or(0), _ => 0 }
        };
        let mut v = [read("MouseThreshold1"), read("MouseThreshold2"), read("MouseSpeed")];
        unsafe {
            SystemParametersInfoW(SPI_SETMOUSE, 0, Some(v.as_mut_ptr().cast()), SPIF_UPDATEINIFILE | SPIF_SENDCHANGE)
                .map_err(|e| SysError::Win32(e.code().0))
        }
    }
}
```

- [ ] **Step 4: Rodar** — `cargo test -p oo-sys` — Expected: todos PASS.
- [ ] **Step 5: Commit** — `git add crates/oo-sys && git commit -m "feat(oo-sys): WindowsBackend real (registro, powercfg, SPI_SETMOUSE)"`

---

### Task 6: `oo-metrics`

**Files:**
- Create: `crates/oo-metrics/Cargo.toml`, `crates/oo-metrics/src/lib.rs`

**Interfaces:**
- Produces: `CpuSampler::new()`, `.sample(&mut self) -> f32` (0–100); `memory() -> (u64 usado, u64 total)`; `timer_resolution_ms() -> f64`; `struct Hardware { cpu: String, gpus: Vec<String>, ram_bytes: u64 }`; `hardware() -> Hardware`

- [ ] **Step 1: `Cargo.toml`**

```toml
[package]
name = "oo-metrics"
version.workspace = true
edition.workspace = true

[dependencies]
serde.workspace = true

[target.'cfg(windows)'.dependencies]
winreg = "0.55"
windows = { version = "0.61", features = ["Win32_Foundation", "Win32_System_Threading", "Win32_System_SystemInformation"] }
```

- [ ] **Step 2: Teste falhando** — fim de `lib.rs`:

```rust
#[cfg(all(test, windows))]
mod tests {
    use super::*;
    #[test]
    fn readings_are_plausible() {
        let mut c = CpuSampler::new();
        std::thread::sleep(std::time::Duration::from_millis(200));
        assert!((0.0..=100.0).contains(&c.sample()));
        let (used, total) = memory();
        assert!(total > 1 << 30 && used <= total);
        let t = timer_resolution_ms();
        assert!(t > 0.0 && t <= 16.0, "timer {t}");
        let h = hardware();
        assert!(!h.cpu.is_empty());
        assert_eq!(h.ram_bytes, total);
    }
}
```
Run: `cargo test -p oo-metrics` — FAIL.

- [ ] **Step 3: Implementar**

```rust
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Hardware { pub cpu: String, pub gpus: Vec<String>, pub ram_bytes: u64 }

#[cfg(windows)]
mod imp {
    use super::Hardware;
    use windows::Win32::Foundation::FILETIME;
    use windows::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
    use windows::Win32::System::Threading::GetSystemTimes;
    use winreg::{enums::HKEY_LOCAL_MACHINE, RegKey};

    fn ft(f: FILETIME) -> u64 { ((f.dwHighDateTime as u64) << 32) | f.dwLowDateTime as u64 }
    fn times() -> (u64, u64) {
        let (mut i, mut k, mut u) = (FILETIME::default(), FILETIME::default(), FILETIME::default());
        unsafe { let _ = GetSystemTimes(Some(&mut i), Some(&mut k), Some(&mut u)); }
        (ft(i), ft(k) + ft(u)) // tempo de kernel já inclui o idle
    }

    pub struct CpuSampler { last: (u64, u64) }
    impl Default for CpuSampler { fn default() -> Self { Self::new() } }
    impl CpuSampler {
        pub fn new() -> Self { Self { last: times() } }
        pub fn sample(&mut self) -> f32 {
            let now = times();
            let (di, dt) = (now.0.saturating_sub(self.last.0), now.1.saturating_sub(self.last.1));
            self.last = now;
            if dt == 0 { 0.0 } else { (100.0 * (1.0 - di as f64 / dt as f64)).clamp(0.0, 100.0) as f32 }
        }
    }

    pub fn memory() -> (u64, u64) {
        let mut m = MEMORYSTATUSEX { dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32, ..Default::default() };
        unsafe { let _ = GlobalMemoryStatusEx(&mut m); }
        (m.ullTotalPhys - m.ullAvailPhys, m.ullTotalPhys)
    }

    #[link(name = "ntdll")]
    extern "system" { fn NtQueryTimerResolution(min: *mut u32, max: *mut u32, cur: *mut u32) -> i32; }

    pub fn timer_resolution_ms() -> f64 {
        let (mut a, mut b, mut c) = (0u32, 0u32, 0u32);
        unsafe { NtQueryTimerResolution(&mut a, &mut b, &mut c); }
        c as f64 / 10_000.0 // unidades de 100 ns
    }

    pub fn hardware() -> Hardware {
        let lm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let cpu = lm.open_subkey(r"HARDWARE\DESCRIPTION\System\CentralProcessor\0")
            .and_then(|k| k.get_value::<String, _>("ProcessorNameString"))
            .map(|s| s.trim().to_string()).unwrap_or_default();
        let mut gpus = vec![];
        if let Ok(cls) = lm.open_subkey(r"SYSTEM\CurrentControlSet\Control\Class\{4d36e968-e325-11ce-bfc1-08002be10318}") {
            for name in cls.enum_keys().flatten().filter(|n| n.chars().all(|c| c.is_ascii_digit())) {
                if let Ok(d) = cls.open_subkey(&name).and_then(|k| k.get_value::<String, _>("DriverDesc")) {
                    if !d.contains("Basic Display") && !gpus.contains(&d) { gpus.push(d); }
                }
            }
        }
        Hardware { cpu, gpus, ram_bytes: memory().1 }
    }
}

#[cfg(windows)]
pub use imp::*;
```

- [ ] **Step 4: Rodar** — `cargo test -p oo-metrics` — PASS.
- [ ] **Step 5: Commit** — `git add crates/oo-metrics && git commit -m "feat(oo-metrics): CPU, memória, timer e hardware reais"`

---

### Task 7: UI scaffold + design system + i18n + format

**Files:**
- Create: `ui/` (Vite svelte-ts), `ui/src/lib/design/{tokens.css,Switch,Button,Card,Ring,Sparkline}.svelte`, `ui/src/lib/i18n/{index,pt-BR,en}.ts`, `ui/src/lib/i18n/i18n.test.ts`, `ui/src/lib/format.ts`, `ui/src/lib/format.test.ts`

**Interfaces:**
- Produces:
  - `<Switch checked disabled size="sm"|"lg" label onchange={(v:boolean)=>void}/>`; `<Button variant="primary"|"ghost" progress={0..1} disabled onclick>`; `<Card>`; `<Ring value={0..100}/>`; `<Sparkline values={number[]} min={number} max={number}/>`
  - `t(key: MessageKey, vars?) => string`; stores `locale`; `setLocale(l: Locale)`; `type Locale = 'pt-BR'|'en'`
  - `formatBytes(n, locale) => string` (GB, 1 casa); `formatMs(n) => string` (2 casas)

- [ ] **Step 1: Scaffold**

```bash
npm create vite@latest ui -- --template svelte-ts
cd ui
npm i @tauri-apps/api@2 @tauri-apps/plugin-updater@2 @tauri-apps/plugin-process@2 @fontsource-variable/geist @fontsource-variable/geist-mono
npm i -D vitest@3 svelte-check
```
Apague `src/lib/Counter.svelte`, `src/assets/`, `src/app.css`, `public/vite.svg`. Scripts em `ui/package.json`: `"check": "svelte-check --tsconfig ./tsconfig.app.json"`, `"test": "vitest run"`, e `"version": "0.1.0"`.

`ui/vite.config.ts`:
```ts
import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,
  server: { port: 1420, strictPort: true },
  build: { target: 'chrome120' },
  test: { environment: 'node' },
});
```

- [ ] **Step 2: Testes falhando**

`src/lib/format.test.ts`:
```ts
import { describe, it, expect } from 'vitest';
import { formatBytes, formatMs } from './format';
describe('format', () => {
  it('GB com 1 casa no separador do idioma', () => {
    expect(formatBytes(11.24 * 1024 ** 3, 'pt-BR')).toBe('11,2');
    expect(formatBytes(11.24 * 1024 ** 3, 'en')).toBe('11.2');
  });
  it('ms com 2 casas', () => expect(formatMs(15.625)).toBe('15.63'));
});
```
`src/lib/i18n/i18n.test.ts`:
```ts
import { it, expect } from 'vitest';
import ptBR from './pt-BR';
import en from './en';
it('en e pt-BR têm exatamente as mesmas chaves', () => {
  expect(Object.keys(en).sort()).toEqual(Object.keys(ptBR).sort());
});
```
Run: `npm test` — FAIL.

- [ ] **Step 3: `format.ts`**

```ts
export const formatBytes = (n: number, locale: string) =>
  (n / 1024 ** 3).toLocaleString(locale, { minimumFractionDigits: 1, maximumFractionDigits: 1 });
export const formatMs = (n: number) => n.toFixed(2);
```

- [ ] **Step 4: `i18n/pt-BR.ts`** (fonte das chaves):

```ts
const ptBR = {
  'nav.overview': 'Visão geral', 'nav.optimize': 'OTIMIZAR', 'nav.system': 'SISTEMA',
  'nav.performance': 'Desempenho', 'nav.gaming': 'Modo Gaming', 'nav.memory': 'Memória',
  'nav.drivers': 'Drivers', 'nav.apps': 'Apps & Ajustes', 'nav.history': 'Histórico', 'nav.settings': 'Configurações',
  'soon': 'Em breve', 'soon.body': 'Esta seção chega numa próxima atualização.',
  'overview.score': 'PONTUAÇÃO', 'overview.recommended': '{n} otimizações recomendadas', 'overview.allDone': 'Tudo em dia',
  'overview.body': 'Menos latência de entrada e frametime mais estável. Cada mudança é registrada e pode ser desfeita.',
  'overview.doneBody': '{n} mudanças aplicadas. Veja ou desfaça em Histórico.',
  'action.optimize': 'Otimizar', 'action.applying': 'Aplicando {i}/{n}', 'action.optimized': 'Otimizado',
  'action.review': 'Revisar', 'action.undo': 'Desfazer', 'action.undoAll': 'Restaurar tudo ao original',
  'action.cancel': 'Cancelar', 'action.continue': 'Continuar', 'action.why': 'Por quê?',
  'metric.cpu': 'CPU', 'metric.memory': 'MEMÓRIA', 'metric.timer': 'TIMER', 'metric.gpu': 'GPU',
  'cat.energy': 'ENERGIA', 'cat.fps': 'FPS', 'cat.input': 'ENTRADA',
  'tweak.power-throttling-off.title': 'Desativar power throttling',
  'tweak.power-throttling-off.desc': 'Evita redução de clock em processos ativos',
  'tweak.power-throttling-off.why': 'Power Throttling reduz a frequência de processos que o Windows considera em segundo plano; jogos sem foco e launchers sofrem quedas.',
  'tweak.mmcss-games.title': 'Prioridade MMCSS para jogos',
  'tweak.mmcss-games.desc': 'Threads do jogo antes de tarefas em segundo plano',
  'tweak.mmcss-games.why': 'MMCSS reserva CPU para multimídia; SystemResponsiveness=10 deixa só 10% para segundo plano e a tarefa Games ganha prioridade alta de CPU e GPU.',
  'tweak.game-dvr-off.title': 'Desativar gravação em segundo plano',
  'tweak.game-dvr-off.desc': 'Libera o encoder de vídeo da GPU durante o jogo',
  'tweak.game-dvr-off.why': 'A gravação em segundo plano do Xbox Game Bar mantém um encoder de vídeo ativo durante o jogo, consumindo GPU e VRAM.',
  'tweak.mouse-accel-off.title': 'Desativar aceleração do mouse',
  'tweak.mouse-accel-off.desc': 'Mesmo movimento físico, mesma distância na tela',
  'tweak.mouse-accel-off.why': '"Aprimorar precisão do ponteiro" aplica curva de aceleração: o mesmo movimento físico gera distâncias diferentes conforme a velocidade.',
  'tweak.ultimate-power-plan.title': 'Plano Desempenho Máximo',
  'tweak.ultimate-power-plan.desc': 'Remove estados de economia que atrasam a resposta da CPU',
  'tweak.ultimate-power-plan.why': 'Duplica e ativa o plano oculto "Desempenho Máximo" do Windows, que elimina microlatências de troca de estado de energia. Desfazer reativa o seu plano anterior.',
  'state.unsupported': 'Não suportado neste PC', 'state.reboot': 'Requer reinício',
  'history.empty': 'Nenhuma mudança ainda.', 'history.Applied': 'Aplicado', 'history.Reverted': 'Desfeito',
  'history.Failed': 'Falhou', 'history.Pending': 'Interrompido',
  'conflict.title': 'Valor alterado por outro programa',
  'conflict.body': 'Desfazer vai sobrescrever o valor atual com o original. Continuar?',
  'pending.banner': 'Uma otimização foi interrompida. Restaurar os valores originais?',
  'update.available': 'Atualização {v} disponível', 'update.install': 'Atualizar', 'update.notes': 'Ver novidades',
  'update.check': 'Verificar atualizações', 'update.upToDate': 'Você está na versão mais recente',
  'update.error': 'Não foi possível verificar atualizações',
  'settings.language': 'Idioma', 'settings.about': 'Sobre', 'settings.version': 'Versão', 'settings.repo': 'Código-fonte no GitHub',
  'snapshot.skipped': 'Ponto de restauração não criado (o histórico continua protegendo as mudanças)',
  'err.AccessDenied': 'Acesso negado. Execute como administrador.', 'err.NotSupported': 'Não suportado neste sistema.',
  'err.Conflict': 'O valor foi alterado por outro programa.', 'err.Io': 'Falha de leitura ou escrita.',
  'err.Win32': 'O Windows recusou a operação (código {code}).', 'err.Internal': 'Erro interno. Veja os logs.',
  'gaming.soon': 'Ativação automática ao abrir jogos chega numa próxima atualização.',
} as const;
export default ptBR;
export type MessageKey = keyof typeof ptBR;
```

`i18n/en.ts`: `import type { MessageKey } from './pt-BR'; const en: Record<MessageKey, string> = { ... }; export default en;` com **cada chave acima** traduzida para inglês (ex.: `'nav.overview': 'Overview'`, `'action.optimize': 'Optimize'`, `'overview.recommended': '{n} recommended optimizations'`, …). O tipo `Record<MessageKey,string>` faz o `svelte-check` falhar se faltar chave; o teste do Step 2 pega chaves extras.

`i18n/index.ts`:
```ts
import { writable, derived } from 'svelte/store';
import ptBR, { type MessageKey } from './pt-BR';
import en from './en';
export type Locale = 'pt-BR' | 'en';
export type { MessageKey };
const dicts: Record<Locale, Record<MessageKey, string>> = { 'pt-BR': ptBR, en };
function initial(): Locale {
  try { const s = localStorage.getItem('oo.locale'); if (s === 'pt-BR' || s === 'en') return s; } catch { /* sem storage */ }
  return navigator.language.toLowerCase().startsWith('pt') ? 'pt-BR' : 'en';
}
export const locale = writable<Locale>(initial());
export function setLocale(l: Locale) { locale.set(l); try { localStorage.setItem('oo.locale', l); } catch { /* ignora */ } }
/** Uso nos componentes: {$t('nav.overview')} — reage à troca de idioma. */
export const t = derived(locale, (l) => (key: MessageKey, vars: Record<string, string | number> = {}) =>
  dicts[l][key].replace(/\{(\w+)\}/g, (_, k: string) => String(vars[k] ?? '')));
```

- [ ] **Step 5: Rodar** — `npm test` — PASS.

- [ ] **Step 6: `tokens.css`** — o bloco `:root` de cores/curvas do mockup v3 (valores das Global Constraints), mais:

```css
@import '@fontsource-variable/geist';
@import '@fontsource-variable/geist-mono';
:root { --font: 'Geist Variable', 'Segoe UI Variable', system-ui, sans-serif; --mono: 'Geist Mono Variable', ui-monospace, monospace; }
*, *::before, *::after { box-sizing: border-box; }
html, body { margin: 0; height: 100%; background: var(--bg); color: var(--ice); font: 13px/1.45 var(--font);
  -webkit-font-smoothing: antialiased; user-select: none; overflow: hidden; }
.mono { font-family: var(--mono); }
.label { font: 500 10px var(--mono); letter-spacing: .14em; color: var(--dim); text-transform: uppercase; }
@keyframes rise { from { opacity: 0; transform: translateY(8px); } to { opacity: 1; transform: none; } }
.rise { animation: rise .6s var(--ease) both; animation-delay: calc(var(--i, 0) * 60ms); }
@media (prefers-reduced-motion: reduce) { *, *::before, *::after { animation: none !important; transition: none !important; } }
```

- [ ] **Step 7: `Switch.svelte`** (correção do bug do mockup: `flex:none` + percurso do knob derivado das medidas do trilho):

```svelte
<script lang="ts">
  type Props = { checked?: boolean; disabled?: boolean; size?: 'sm' | 'lg'; label: string; onchange?: (v: boolean) => void };
  let { checked = false, disabled = false, size = 'sm', label, onchange }: Props = $props();
  const dims = { sm: { w: 34, h: 20 }, lg: { w: 46, h: 26 } } as const;
  const PAD = 3;
  const d = $derived(dims[size]);
  const knob = $derived(d.h - PAD * 2);
  const travel = $derived(d.w - PAD * 2 - knob);
</script>

<button type="button" role="switch" aria-checked={checked} aria-label={label} {disabled}
  class="sw" class:on={checked}
  style="--w:{d.w}px;--h:{d.h}px;--pad:{PAD}px;--k:{knob}px;--travel:{travel}px"
  onclick={() => onchange?.(!checked)}><span class="knob"></span></button>

<style>
  .sw { flex: none; position: relative; width: var(--w); height: var(--h); padding: 0; border: 0; border-radius: 999px;
        background: #202227; cursor: pointer; transition: background .25s var(--ease); }
  .sw:disabled { opacity: .4; cursor: default; }
  .sw.on { background: var(--ice); }
  .knob { position: absolute; top: var(--pad); left: var(--pad); width: var(--k); height: var(--k); border-radius: 50%;
          background: var(--mut); transition: transform .42s var(--spring), background .25s; }
  .sw.on .knob { transform: translateX(var(--travel)); background: var(--bg); }
  .sw:focus-visible { outline: 1.5px solid var(--ice); outline-offset: 2px; }
</style>
```

- [ ] **Step 8: `Button`, `Card`, `Ring`, `Sparkline`** — porte os estilos `.btn/.ghost`, `.card`, `.ring/.arc`, `.spark` de `docs/design/mockup-glacier-v3.html`:
  - `Ring.svelte`: `let { value }: { value: number } = $props();` círculo r=70, `stroke-dasharray=439.8`, `stroke-dashoffset = 439.8 * (1 - value/100)` com `transition: stroke-dashoffset 1.4s var(--ease)`; número exibido animado com `$effect` + `requestAnimationFrame`, easing `1 - (1-p)**3`, partindo do valor exibido atual; legenda `{$t('overview.score')}`.
  - `Sparkline.svelte`: `let { values, min, max }` → `path` linha e área (`d + ' L100 26 L0 26Z'`), `viewBox="0 0 100 26" preserveAspectRatio="none"`, `<linearGradient>` com `id` único por instância (`$props.id()`).
  - `Button.svelte`: `let { variant = 'primary', progress = 0, disabled = false, onclick, children }` → `<button>` com `<i class="fill" style="width:{progress*100}%">` e `{@render children()}`.
  - `Card.svelte`: `<div class="card {klass}" style={style}>{@render children()}</div>`.

- [ ] **Step 9: `npm run check`** — Expected: 0 errors, 0 warnings.
- [ ] **Step 10: Commit** — `git add ui && git commit -m "feat(ui): scaffold Svelte 5, design system Glacier e i18n pt-BR/en"`

---

### Task 8: App Tauri (bootstrap, manifest admin, comandos IPC, bindings ts-rs, ponto de restauração)

**Files:**
- Create: `package.json` (raiz), `app/{Cargo.toml,build.rs,app.manifest,tauri.conf.json}`, `app/capabilities/default.json`, `app/src/{main,commands,views,restore}.rs`, `ui/src/lib/ipc/api.ts`, `docs/design/icon.svg`
- Modify: `Cargo.toml` (`members = ["crates/*", "app"]`)

**Interfaces:**
- Consumes: Tasks 3–6
- Produces (comandos; nomes exatos): `hardware`, `metrics`, `tweaks`, `score`, `apply_tweaks(ids)`, `revert_tweak(id, force)`, `revert_change(id, force)`, `revert_all(force)`, `history`, `pending`; evento `apply-progress`. Tipos TS em `ui/src/lib/ipc/bindings/`: `HardwareView`, `MetricsView`, `TweakView`, `ChangeView`, `ErrorView`, `ApplyResult`, `BatchResult`, `Progress`. Objeto `api` em `ui/src/lib/ipc/api.ts`.

- [ ] **Step 1: Raiz** — `package.json`: `{ "private": true, "scripts": { "tauri": "tauri" }, "devDependencies": { "@tauri-apps/cli": "^2" } }` → `npm i`. Em `Cargo.toml`, `members = ["crates/*", "app"]`.

- [ ] **Step 2: `app/Cargo.toml`**

```toml
[package]
name = "open-optimizer"
version.workspace = true
edition.workspace = true

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = [] }
tauri-plugin-updater = "2"
tauri-plugin-process = "2"
serde.workspace = true
serde_json.workspace = true
ts-rs = "10"
oo-core = { path = "../crates/oo-core" }
oo-sys = { path = "../crates/oo-sys" }
oo-catalog = { path = "../crates/oo-catalog" }
oo-metrics = { path = "../crates/oo-metrics" }
```

- [ ] **Step 3: Manifest + `build.rs`**

`app/app.manifest`:
```xml
<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <dependency><dependentAssembly>
    <assemblyIdentity type="win32" name="Microsoft.Windows.Common-Controls" version="6.0.0.0"
      processorArchitecture="*" publicKeyToken="6595b64144ccf1df" language="*"/>
  </dependentAssembly></dependency>
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3"><security><requestedPrivileges>
    <requestedExecutionLevel level="requireAdministrator" uiAccess="false"/>
  </requestedPrivileges></security></trustInfo>
</assembly>
```
`app/build.rs`:
```rust
fn main() {
    let attrs = tauri_build::Attributes::new()
        .windows_attributes(tauri_build::WindowsAttributes::new().app_manifest(include_str!("app.manifest")));
    tauri_build::try_build(attrs).expect("tauri build");
}
```

- [ ] **Step 4: Ícone + `tauri.conf.json` + capabilities**

`docs/design/icon.svg` (símbolo `.mark` do mockup em 1024 px):
```svg
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1024 1024">
  <rect width="1024" height="1024" rx="232" fill="#EEF3F7"/>
  <g transform="translate(512 512) scale(24)" fill="none" stroke="#0B0C0E" stroke-width="2.6" stroke-linecap="round">
    <path d="M0 -10v20M-8.5 -5l17 10M8.5 -5l-17 10"/>
  </g>
</svg>
```
Run: `npx tauri icon docs/design/icon.svg -o app/icons`

`app/tauri.conf.json`:
```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "Open Optimizer",
  "version": "0.1.0",
  "identifier": "io.github.lucianobarral.openoptimizer",
  "build": {
    "frontendDist": "../ui/dist",
    "devUrl": "http://localhost:1420",
    "beforeDevCommand": { "script": "npm run dev", "cwd": "../ui" },
    "beforeBuildCommand": { "script": "npm run build", "cwd": "../ui" }
  },
  "app": {
    "windows": [{ "label": "main", "title": "Open Optimizer", "width": 1180, "height": 760, "minWidth": 1100, "minHeight": 700,
                  "decorations": false, "backgroundColor": "#0B0C0E", "center": true }],
    "security": { "csp": "default-src 'self'; style-src 'self' 'unsafe-inline'; font-src 'self'; img-src 'self' data:; connect-src ipc: http://ipc.localhost" }
  },
  "bundle": {
    "active": true, "targets": ["nsis"], "createUpdaterArtifacts": true,
    "icon": ["icons/32x32.png", "icons/128x128.png", "icons/icon.ico"],
    "windows": { "nsis": { "installMode": "perMachine" }, "webviewInstallMode": { "type": "downloadBootstrapper" } }
  },
  "plugins": {
    "updater": {
      "endpoints": ["https://github.com/LucianoBarral/OpenOptimizer/releases/latest/download/latest.json"],
      "pubkey": "PREENCHIDO_NA_TASK_10_STEP_1",
      "windows": { "installMode": "passive" }
    }
  }
}
```
(O `pubkey` é a única pendência intencional e é resolvida no primeiro passo da Task 10, antes de qualquer release.)

`app/capabilities/default.json`:
```json
{ "identifier": "default", "windows": ["main"],
  "permissions": ["core:default", "core:window:allow-minimize", "core:window:allow-toggle-maximize",
                  "core:window:allow-close", "core:window:allow-start-dragging", "updater:default", "process:allow-restart"] }
```

- [ ] **Step 5: `views.rs`**

```rust
use serde::Serialize;
use ts_rs::TS;

macro_rules! view { ($($i:item)*) => { $(
    #[derive(Debug, Clone, Serialize, TS)]
    #[ts(export, export_to = "../../ui/src/lib/ipc/bindings/")]
    $i
)* } }

view! {
    pub struct HardwareView { pub cpu: String, pub gpus: Vec<String>, pub ram_bytes: f64 }
    pub struct MetricsView { pub cpu: f32, pub mem_used: f64, pub mem_total: f64, pub timer_ms: f64 }
    pub struct TweakView { pub id: String, pub category: String, pub requires_reboot: bool, pub state: String }
    pub struct ChangeView { pub id: String, pub tweak_id: String, pub batch_id: String, pub timestamp: String, pub status: String }
    pub struct ErrorView { pub kind: String, pub code: Option<i32>, pub change_id: Option<String> }
    pub struct ApplyResult { pub tweak_id: String, pub ok: bool, pub error: Option<ErrorView> }
    pub struct BatchResult { pub batch_id: String, pub restore_point: bool, pub results: Vec<ApplyResult> }
    pub struct Progress { pub index: u32, pub total: u32, pub tweak_id: String }
}

impl From<&oo_core::OoError> for ErrorView {
    fn from(e: &oo_core::OoError) -> Self {
        use oo_core::OoError::*;
        use oo_sys::SysError as S;
        let (kind, code, change_id) = match e {
            Sys(S::AccessDenied) => ("AccessDenied", None, None),
            Sys(S::NotFound) | NotFound(_) => ("NotSupported", None, None),
            Sys(S::Win32(c)) => ("Win32", Some(*c), None),
            Sys(S::Io(_)) | Io(_) => ("Io", None, None),
            Conflict { change_id } => ("Conflict", None, Some(change_id.clone())),
        };
        ErrorView { kind: kind.into(), code, change_id }
    }
}
```
(`ts-rs` gera os `.ts` ao rodar `cargo test -p open-optimizer`; não é preciso teste manual.)

- [ ] **Step 6: `restore.rs`**

```rust
use std::os::windows::process::CommandExt;

/// Best-effort: false se o Windows recusar (limite de 1 por 24 h, proteção desligada).
pub fn create_restore_point() -> bool {
    std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command",
               "Checkpoint-Computer -Description 'Open Optimizer' -RestorePointType MODIFY_SETTINGS -ErrorAction Stop"])
        .creation_flags(0x0800_0000)
        .output().map(|o| o.status.success()).unwrap_or(false)
}
```

- [ ] **Step 7: `commands.rs`**

```rust
use crate::views::*;
use oo_core::{engine::new_batch_id, Change, Engine, TweakDef};
use oo_sys::windows::WindowsBackend;
use std::sync::Mutex;
use tauri::{Emitter, State};

pub struct AppState {
    pub engine: Mutex<Engine<WindowsBackend>>,
    pub catalog: Vec<TweakDef>,
    pub cpu: Mutex<oo_metrics::CpuSampler>,
    pub restore_attempted: Mutex<bool>,
}

#[tauri::command]
pub fn hardware() -> HardwareView {
    let h = oo_metrics::hardware();
    HardwareView { cpu: h.cpu, gpus: h.gpus, ram_bytes: h.ram_bytes as f64 }
}

#[tauri::command]
pub fn metrics(s: State<AppState>) -> MetricsView {
    let (used, total) = oo_metrics::memory();
    MetricsView { cpu: s.cpu.lock().unwrap().sample(), mem_used: used as f64, mem_total: total as f64,
                  timer_ms: oo_metrics::timer_resolution_ms() }
}

#[tauri::command]
pub fn tweaks(s: State<AppState>) -> Vec<TweakView> {
    let e = s.engine.lock().unwrap();
    s.catalog.iter().map(|t| TweakView { id: t.id.clone(), category: t.category.clone(),
        requires_reboot: t.requires_reboot, state: format!("{:?}", e.detect(t)) }).collect()
}

#[tauri::command]
pub fn score(s: State<AppState>) -> u8 { s.engine.lock().unwrap().score(&s.catalog) }

#[tauri::command]
pub async fn apply_tweaks(app: tauri::AppHandle, s: State<'_, AppState>, ids: Vec<String>) -> Result<BatchResult, ()> {
    let mut restore_point = true;
    {
        let mut tried = s.restore_attempted.lock().unwrap();
        if !*tried { restore_point = crate::restore::create_restore_point(); *tried = true; }
    }
    let batch_id = new_batch_id();
    let total = ids.len() as u32;
    let mut results = Vec::with_capacity(ids.len());
    for (i, id) in ids.iter().enumerate() {
        let _ = app.emit("apply-progress", Progress { index: i as u32 + 1, total, tweak_id: id.clone() });
        let r = match s.catalog.iter().find(|t| &t.id == id) {
            Some(t) => s.engine.lock().unwrap().apply(t, &batch_id).map(|_| ()),
            None => Err(oo_core::OoError::NotFound(id.clone())),
        };
        results.push(ApplyResult { tweak_id: id.clone(), ok: r.is_ok(), error: r.err().as_ref().map(ErrorView::from) });
    }
    Ok(BatchResult { batch_id, restore_point, results })
}

#[tauri::command]
pub fn revert_tweak(s: State<AppState>, id: String, force: bool) -> Result<(), ErrorView> {
    s.engine.lock().unwrap().revert_tweak(&id, force).map_err(|e| (&e).into())
}

#[tauri::command]
pub fn revert_change(s: State<AppState>, id: String, force: bool) -> Result<(), ErrorView> {
    s.engine.lock().unwrap().revert(&id, force).map_err(|e| (&e).into())
}

#[tauri::command]
pub fn revert_all(s: State<AppState>, force: bool) -> Vec<ErrorView> {
    s.engine.lock().unwrap().revert_all(force).iter().map(|(_, e)| e.into()).collect()
}

fn view(c: Change) -> ChangeView {
    ChangeView { id: c.id, tweak_id: c.tweak_id, batch_id: c.batch_id, timestamp: c.timestamp, status: format!("{:?}", c.status) }
}

#[tauri::command]
pub fn history(s: State<AppState>) -> Vec<ChangeView> {
    s.engine.lock().unwrap().history().into_iter().rev().map(view).collect()
}

#[tauri::command]
pub fn pending(s: State<AppState>) -> Vec<ChangeView> {
    s.engine.lock().unwrap().pending().into_iter().map(view).collect()
}
```

- [ ] **Step 8: `main.rs`**

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod commands;
mod restore;
mod views;

use std::path::PathBuf;
use std::sync::Mutex;

fn data_dir() -> PathBuf {
    PathBuf::from(std::env::var("ProgramData").unwrap_or_else(|_| r"C:\ProgramData".into())).join("OpenOptimizer")
}

fn main() {
    let dir = data_dir();
    let logs = dir.join("logs");
    let _ = std::fs::create_dir_all(&logs);
    // panic = "abort" no release: grava o motivo antes de encerrar.
    std::panic::set_hook(Box::new(move |info| {
        let _ = std::fs::write(logs.join("panic.log"), format!("{info}\n{}", std::backtrace::Backtrace::force_capture()));
    }));
    let journal = oo_core::Journal::open(dir.join("journal.jsonl")).expect("não foi possível abrir o journal");
    let state = commands::AppState {
        engine: Mutex::new(oo_core::Engine::new(oo_sys::windows::WindowsBackend::new(), journal)),
        catalog: oo_catalog::load_catalog(),
        cpu: Mutex::new(oo_metrics::CpuSampler::new()),
        restore_attempted: Mutex::new(false),
    };
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::hardware, commands::metrics, commands::tweaks, commands::score, commands::apply_tweaks,
            commands::revert_tweak, commands::revert_change, commands::revert_all, commands::history, commands::pending
        ])
        .run(tauri::generate_context!())
        .expect("falha ao iniciar o Tauri");
}
```

- [ ] **Step 9: Gerar bindings** — Run: `cargo test -p open-optimizer` — Expected: 8 arquivos em `ui/src/lib/ipc/bindings/`.

- [ ] **Step 10: `ui/src/lib/ipc/api.ts`**

```ts
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { HardwareView } from './bindings/HardwareView';
import type { MetricsView } from './bindings/MetricsView';
import type { TweakView } from './bindings/TweakView';
import type { ChangeView } from './bindings/ChangeView';
import type { BatchResult } from './bindings/BatchResult';
import type { ErrorView } from './bindings/ErrorView';
import type { Progress } from './bindings/Progress';

export const api = {
  hardware: () => invoke<HardwareView>('hardware'),
  metrics: () => invoke<MetricsView>('metrics'),
  tweaks: () => invoke<TweakView[]>('tweaks'),
  score: () => invoke<number>('score'),
  applyTweaks: (ids: string[]) => invoke<BatchResult>('apply_tweaks', { ids }),
  revertTweak: (id: string, force = false) => invoke<void>('revert_tweak', { id, force }),
  revertChange: (id: string, force = false) => invoke<void>('revert_change', { id, force }),
  revertAll: (force = false) => invoke<ErrorView[]>('revert_all', { force }),
  history: () => invoke<ChangeView[]>('history'),
  pending: () => invoke<ChangeView[]>('pending'),
  onProgress: (cb: (p: Progress) => void) => listen<Progress>('apply-progress', (e) => cb(e.payload)),
};
export const isErrorView = (e: unknown): e is ErrorView => typeof e === 'object' && e !== null && 'kind' in e;
export type { HardwareView, MetricsView, TweakView, ChangeView, BatchResult, ErrorView, Progress };
```

- [ ] **Step 11: Build** — terminal **admin**: `npm run tauri build -- --no-bundle` — Expected: compila. Depois `cargo clippy --workspace -- -D warnings` limpo e `cd ui && npm run check` limpo.

- [ ] **Step 12: Commit** — `git add -A package.json package-lock.json Cargo.toml app ui/src/lib/ipc docs/design/icon.svg && git commit -m "feat(app): Tauri com manifest admin, comandos IPC e bindings ts-rs"`

---

### Task 9: Telas

**Files:**
- Create: `ui/src/App.svelte` (substituir), `ui/src/main.ts` (substituir), `ui/src/lib/stores.ts`, `ui/src/lib/shell/{TitleBar,Sidebar,ConfirmDialog}.svelte`, `ui/src/routes/{Overview,Performance,History,Settings,ComingSoon}.svelte`, `ui/src/lib/tweakRow/TweakRow.svelte`

**Interfaces:**
- Consumes: `api`, `isErrorView` (Task 8); design + `t` (Task 7)
- Produces: `stores.ts` → `type Route`, `route`, `tweaks`, `score`, `refresh()`; `TweakRow.svelte` (props `tweak: TweakView`, `expandable?: boolean`) reutilizado por Overview e Performance; `ConfirmDialog.svelte` (props `title`, `body`, `onconfirm`, `oncancel`)

- [ ] **Step 1: `main.ts` + `stores.ts`**

```ts
// main.ts
import './lib/design/tokens.css';
import { mount } from 'svelte';
import App from './App.svelte';
export default mount(App, { target: document.getElementById('app')! });
```
```ts
// lib/stores.ts
import { writable } from 'svelte/store';
import { api, type TweakView } from './ipc/api';
export type Route = 'overview' | 'performance' | 'gaming' | 'memory' | 'drivers' | 'apps' | 'history' | 'settings';
export const route = writable<Route>('overview');
export const tweaks = writable<TweakView[]>([]);
export const score = writable(0);
export const applying = writable(false); // bloqueia update durante batch
export async function refresh() {
  const [t, s] = await Promise.all([api.tweaks(), api.score()]);
  tweaks.set(t); score.set(s);
}
```

- [ ] **Step 2: `TitleBar.svelte`** — `<header data-tauri-drag-region>` de 36 px, `border-bottom: 1px solid var(--line)`, texto `open optimizer` (`.mono`, `--dim`), três botões 46×36 à direita (traço, quadrado vazado, ×) chamando `getCurrentWindow().minimize() / toggleMaximize() / close()` de `@tauri-apps/api/window`; hover `background: #17191D`, e o fechar com hover `#2A1416`.

- [ ] **Step 3: `ConfirmDialog.svelte`** — overlay `rgba(0,0,0,.5)` com fade, card central (`--card`, 1px `--line-2`, raio 12) com entrada `scale(.98)→1` + `opacity`, botões `Button ghost` (`action.cancel`) e `Button primary` (`action.continue`); `Esc` cancela; foco inicial no botão de cancelar.

- [ ] **Step 4: `TweakRow.svelte`**

```svelte
<script lang="ts">
  import Switch from '../design/Switch.svelte';
  import ConfirmDialog from '../shell/ConfirmDialog.svelte';
  import { api, isErrorView, type TweakView, type ErrorView } from '../ipc/api';
  import { refresh, applying } from '../stores';
  import { t } from '../i18n';
  import type { MessageKey } from '../i18n';
  let { tweak, expandable = false }: { tweak: TweakView; expandable?: boolean } = $props();
  let error = $state<ErrorView | null>(null);
  let conflict = $state(false);
  let open = $state(false);
  const k = (s: string) => s as MessageKey;
  const on = $derived(tweak.state === 'Applied');
  const unsupported = $derived(tweak.state === 'Unsupported');

  async function toggle(v: boolean, force = false) {
    error = null;
    try {
      if (v) {
        applying.set(true);
        const r = await api.applyTweaks([tweak.id]);
        const res = r.results[0];
        if (!res.ok && res.error) error = res.error;
      } else {
        await api.revertTweak(tweak.id, force);
      }
    } catch (e) {
      if (isErrorView(e) && e.kind === 'Conflict') conflict = true;
      else if (isErrorView(e)) error = e;
    } finally { applying.set(false); await refresh(); }
  }
</script>

<div class="row" class:is-on={on}>
  <div class="txt">
    <div class="ttl">{$t(k(`tweak.${tweak.id}.title`))}</div>
    <div class="desc">{unsupported ? $t('state.unsupported') : $t(k(`tweak.${tweak.id}.desc`))}
      {#if tweak.requires_reboot}<span class="chip">{$t('state.reboot')}</span>{/if}</div>
    {#if error}<div class="err">{$t(k(`err.${error.kind}`), { code: error.code ?? '' })}</div>{/if}
    {#if expandable}
      <button class="why" onclick={() => (open = !open)}>{$t('action.why')}</button>
      {#if open}<p class="rationale">{$t(k(`tweak.${tweak.id}.why`))}</p>{/if}
    {/if}
  </div>
  <span class="tag mono">{$t(k(`cat.${tweak.category}`))}</span>
  <Switch checked={on} disabled={unsupported} label={$t(k(`tweak.${tweak.id}.title`))} onchange={(v) => toggle(v)} />
</div>
{#if conflict}
  <ConfirmDialog title={$t('conflict.title')} body={$t('conflict.body')}
    onconfirm={() => { conflict = false; toggle(false, true); }} oncancel={() => (conflict = false)} />
{/if}

<style>
  .row { display: grid; grid-template-columns: 1fr auto 36px; gap: 14px; align-items: center; padding: 13px 0; border-bottom: 1px solid var(--line); }
  .row:last-child { border: 0; }
  .ttl { font-size: 13.5px; } .desc { color: var(--mut); font-size: 12px; margin-top: 1px; }
  .tag { font-size: 10.5px; color: var(--mut); letter-spacing: .04em; }
  .err { color: #E5A5A5; font-size: 12px; margin-top: 4px; }
  .chip { margin-left: 6px; border: 1px solid var(--line-2); border-radius: 5px; padding: 0 5px; font-size: 11px; }
  .why { background: none; border: 0; padding: 0; margin-top: 6px; color: var(--ice-2); font: inherit; font-size: 12px; cursor: pointer; }
  .rationale { color: var(--mut); font-size: 12px; line-height: 1.55; margin: 6px 0 0; max-width: 560px; animation: rise .3s var(--ease); }
</style>
```

- [ ] **Step 5: `Sidebar.svelte`** — porte o `aside` do mockup v3 (marca, grupos, ícones SVG, rodapé). Itens: `overview`, grupo OTIMIZAR (`performance`, `gaming`, `memory`), grupo SISTEMA (`drivers`, `apps`, `history`), e `settings` ao final. Indicador `.ind` absoluto que desliza: `$effect` lê o `offsetTop` do item ativo (via `bind:this` num mapa de elementos) e aplica `style:transform={`translateY(${y}px)`}` com `transition: transform .38s var(--ease)`. Badge de Desempenho = `$tweaks.filter(x => x.state === 'NotApplied' || x.state === 'Partial').length` (oculto se 0). Itens `gaming|memory|drivers|apps` exibem selo `$t('soon')`. Rodapé: `v{version}` de `getVersion()`.

- [ ] **Step 6: `Overview.svelte`**
  - `onMount`: `api.hardware()` → subtítulo `${cpu} · ${gpus[0] ?? ''} · ${formatBytes(ram_bytes, $locale)} GB`; `refresh()`; `api.pending()` → se houver itens, banner (`pending.banner`) com botão que faz `for (p of pend) await api.revertChange(p.id, true)` e depois `refresh()`.
  - Métricas: `setInterval` de 1000 ms que só chama `api.metrics()` se `document.visibilityState === 'visible'`; guarda os últimos 24 pontos de `cpu`, `mem_used/2**30` e `timer_ms` para os `Sparkline` (min/max: 0–100, 0–`mem_total/2**30`, 0–16). Limpa o intervalo no `onDestroy`.
  - Cards de métrica (grade 2×2): CPU `%`, Memória `formatBytes` GB, Timer `formatMs` ms, GPU = nome `gpus[0]` (sem sparkline e sem número inventado).
  - Hero: `<Ring value={$score}/>`; `pendingCount = $tweaks.filter(NotApplied|Partial).length`; título `overview.recommended` (n=pendingCount) ou `overview.allDone` se 0.
  - Botão Otimizar (`Button primary progress={p}`), desabilitado se `pendingCount === 0` ou `$applying`. Ao clicar: `applying.set(true)`; `const un = await api.onProgress(x => { p = x.index / x.total; label = $t('action.applying', { i: x.index, n: x.total }) })`; `r = await api.applyTweaks(ids)`; `un()`; `refresh()`; `label = $t('action.optimized')`; `doneN = r.results.filter(x => x.ok).length`; se `!r.restore_point` mostrar nota discreta `snapshot.skipped`; `applying.set(false)`.
  - Lista: `Card` com `{#each $tweaks.slice(0, 4) as tw (tw.id)}<TweakRow tweak={tw}/>{/each}`.
  - Card Modo Gaming: `.label` `nav.gaming`, título `$t('soon')`, texto `gaming.soon`, `<Switch size="lg" disabled label={$t('nav.gaming')}/>`.
  - Entrada: cada bloco com `class="rise" style="--i:N"`.

- [ ] **Step 7: `Performance.svelte`** — agrupa `$tweaks` por `category` na ordem `fps`, `energy`, `input`; cada grupo um `Card` com `.label` `cat.*` e `<TweakRow tweak={tw} expandable/>`.

- [ ] **Step 8: `History.svelte`** — `onMount` → `api.history()`; agrupa por `batch_id` preservando a ordem; cabeçalho do grupo com `new Date(timestamp).toLocaleString($locale)`; linha: título do tweak (`tweak.<id>.title`), status `history.<status>`, botão `action.undo` se status `Applied|Pending` → `api.revertChange(id)`; `Conflict` → `ConfirmDialog` → `api.revertChange(id, true)`; recarrega a lista e `refresh()`. Topo: `Button ghost` `action.undoAll` → `errs = await api.revertAll()`; se `errs.some(e => e.kind === 'Conflict')` → `ConfirmDialog` → `api.revertAll(true)`. Vazio → `history.empty`.

- [ ] **Step 9: `Settings.svelte` + `ComingSoon.svelte` + `App.svelte`**
  - Settings: controle segmentado pt-BR / English (`setLocale`), versão (`getVersion()`), seção de atualização (ligada na Task 10), link `settings.repo` abrindo `https://github.com/LucianoBarral/OpenOptimizer` via `<a target="_blank">`.
  - ComingSoon: prop `title: MessageKey` → título + `soon.body`.
  - App: `<div class="app">` em grid `grid-template: 36px 1fr / 224px 1fr`; `TitleBar` ocupa a linha 1 inteira; `Sidebar`; `<main>` com `{#key $route}` (re-dispara a animação `rise`) e roteia: `overview→Overview`, `performance→Performance`, `history→History`, `settings→Settings`, demais → `ComingSoon`.

- [ ] **Step 10: Verificação ao vivo (observada)** — terminal admin: `npm run tauri dev`. Conferir e anotar:
  1. Métricas mudam a cada ~1 s; CPU acompanha a do Gerenciador de Tarefas (±5 pontos).
  2. Minimizar a janela por 10 s → no devtools (Network/Console), nenhuma chamada `metrics` nesse intervalo.
  3. Ligar "Desativar aceleração do mouse" → `reg query "HKCU\Control Panel\Mouse" /v MouseSpeed` = `0`; desligar → volta ao valor anotado antes.
  4. Toggle em 1100×700 e maximizado: knob sempre dentro do trilho (o mesmo teste de bounding box usado no mockup).
  5. Trocar idioma em Configurações → todos os textos mudam na hora.
  `npm run check` e `npm test` verdes.

- [ ] **Step 11: Commit** — `git add ui && git commit -m "feat(ui): telas Visão geral, Desempenho, Histórico e Configurações"`

---

### Task 10: Auto-update, CI e Release

**Files:**
- Create: `ui/src/lib/update.ts`, `ui/src/lib/shell/UpdateBanner.svelte`, `.github/workflows/ci.yml`, `.github/workflows/release.yml`, `scripts/bump-version.mjs`, `README.md`
- Modify: `app/tauri.conf.json` (`pubkey`), `ui/src/App.svelte`, `ui/src/routes/Settings.svelte`

**Interfaces:**
- Produces: `update.ts` → `available` (`writable<Update|null>`), `checkForUpdate(): Promise<'available'|'none'|'error'>`, `installAndRelaunch(u, onProgress)`, `startAutoCheck()`

- [ ] **Step 1: Chaves de assinatura (fora do repositório)**

Run: `npx tauri signer generate -w "$env:USERPROFILE\.tauri\openoptimizer.key"` (definir senha).
Copiar o conteúdo de `openoptimizer.key.pub` para `plugins.updater.pubkey`.
**Pedir confirmação ao usuário** e então gravar os secrets no GitHub:
```bash
gh secret set TAURI_SIGNING_PRIVATE_KEY < "$USERPROFILE/.tauri/openoptimizer.key"
gh secret set TAURI_SIGNING_PRIVATE_KEY_PASSWORD
```
Avisar o usuário para guardar backup da chave privada: sem ela não é possível publicar updates para quem já instalou.

- [ ] **Step 2: `update.ts`**

```ts
import { check, type Update } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';
import { writable } from 'svelte/store';

export const available = writable<Update | null>(null);

export async function checkForUpdate(): Promise<'available' | 'none' | 'error'> {
  try { const u = await check(); available.set(u); return u ? 'available' : 'none'; }
  catch { return 'error'; }
}

export async function installAndRelaunch(u: Update, onProgress: (p: number) => void) {
  let total = 0, got = 0;
  await u.downloadAndInstall((e) => {
    if (e.event === 'Started') total = e.data.contentLength ?? 0;
    if (e.event === 'Progress') { got += e.data.chunkLength; if (total) onProgress(got / total); }
  });
  await relaunch();
}

export function startAutoCheck() {
  void checkForUpdate();
  setInterval(() => void checkForUpdate(), 6 * 60 * 60 * 1000);
}
```

- [ ] **Step 3: `UpdateBanner.svelte`** — visível quando `$available`: linha com `.label` + `update.available` (`v = $available.version`), `Button ghost` `update.notes` (expande `$available.body`), `Button primary` `update.install` com `progress`, **desabilitado enquanto `$applying`**; entrada com `rise`. Em `App.svelte`: `onMount(startAutoCheck)` e `<UpdateBanner/>` no topo do `<main>`. Em Settings: botão `update.check` → `checkForUpdate()` e mostra `update.upToDate` / `update.error` / abre o banner.

- [ ] **Step 4: `scripts/bump-version.mjs`**

```js
// uso: node scripts/bump-version.mjs 0.1.1
import fs from 'node:fs';
const v = process.argv[2];
if (!/^\d+\.\d+\.\d+$/.test(v ?? '')) { console.error('uso: bump-version.mjs X.Y.Z'); process.exit(1); }
const json = (p, f) => { const o = JSON.parse(fs.readFileSync(p, 'utf8')); f(o); fs.writeFileSync(p, JSON.stringify(o, null, 2) + '\n'); };
json('app/tauri.conf.json', (o) => { o.version = v; });
json('ui/package.json', (o) => { o.version = v; });
const cargo = fs.readFileSync('Cargo.toml', 'utf8');
const next = cargo.replace(/(\[workspace\.package\]\s*\nversion = ")[^"]+/, `$1${v}`);
if (next === cargo) { console.error('versão do workspace não encontrada no Cargo.toml'); process.exit(1); }
fs.writeFileSync('Cargo.toml', next);
console.log(`versão → ${v}`);
```
Run: `node scripts/bump-version.mjs 0.1.0` — Expected: `versão → 0.1.0`, `git diff` vazio (já é 0.1.0).

- [ ] **Step 5: `.github/workflows/ci.yml`**

```yaml
name: CI
on:
  pull_request:
  push: { branches: [main] }
jobs:
  check:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with: { components: "rustfmt, clippy" }
      - uses: Swatinem/rust-cache@v2
      - uses: actions/setup-node@v4
        with: { node-version: 24, cache: npm, cache-dependency-path: ui/package-lock.json }
      - run: npm ci
        working-directory: ui
      - run: npm run build
        working-directory: ui
      - run: cargo fmt --all --check
      - run: cargo clippy --workspace -- -D warnings
      - run: cargo test --workspace
      - run: npm run check
        working-directory: ui
      - run: npm test
        working-directory: ui
```
(O runner `windows-latest` roda como administrador, então os testes reais de HKCU e `powercfg` funcionam.)

- [ ] **Step 6: `.github/workflows/release.yml`**

```yaml
name: Release
on:
  push: { tags: ["v*"] }
permissions: { contents: write }
jobs:
  release:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - uses: actions/setup-node@v4
        with: { node-version: 24 }
      - run: npm ci
      - run: npm ci
        working-directory: ui
      - uses: tauri-apps/tauri-action@v0
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          TAURI_SIGNING_PRIVATE_KEY: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY }}
          TAURI_SIGNING_PRIVATE_KEY_PASSWORD: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY_PASSWORD }}
        with:
          projectPath: app
          tagName: ${{ github.ref_name }}
          releaseName: Open Optimizer ${{ github.ref_name }}
          releaseBody: "Veja as mudanças nos commits desta versão."
          releaseDraft: false
          includeUpdaterJson: true
      - name: Tamanho do instalador
        shell: pwsh
        run: Get-ChildItem -Recurse target/release/bundle/nsis/*.exe | ForEach-Object { "{0}: {1:N1} MB" -f $_.Name, ($_.Length / 1MB) }
```

- [ ] **Step 7: README** — o que é; princípios (reversível, sem ban: nunca toca processos de jogo, sem placebo, dados reais); instalar (Releases); compilar (Rust + Node 24, terminal admin, `npm i && npm run tauri dev`); licença MIT (criar `LICENSE`).

- [ ] **Step 8: Commit + push + CI** — `git add -A && git commit -m "feat: auto-update via GitHub Releases, CI e README"` → `git push` → `gh run watch` — Expected: `check` verde. Se vermelho, corrigir antes de seguir.

- [ ] **Step 9: Release v0.1.0** — `git tag v0.1.0 && git push origin v0.1.0` → `gh run watch` → `gh release view v0.1.0` — Expected: assets `*_x64-setup.exe`, `*.sig` e `latest.json`. Registrar o tamanho do instalador do log (≤ 15 MB).

- [ ] **Step 10: Verificação fim a fim (manual e observada; registrar números)**
  1. Antes de instalar, anotar: `reg query "HKLM\SYSTEM\CurrentControlSet\Control\Power\PowerThrottling"`, `reg query "HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile" /s`, `powercfg /getactivescheme`, `reg query "HKCU\System\GameConfigStore" /v GameDVR_Enabled`, `reg query "HKCU\Control Panel\Mouse"`.
  2. Instalar a v0.1.0 → Visão geral → Otimizar → reexecutar os comandos: valores novos conforme `tweaks.toml`; plano ativo = "Desempenho Máximo".
  3. Histórico → Restaurar tudo → comandos idênticos ao passo 1.
  4. Recuperação: Otimizar e, durante "Aplicando", `taskkill /f /im open-optimizer.exe`; reabrir → banner `pending.banner` → restaurar → valores do passo 1.
  5. RAM em repouso: janela aberta 60 s; somar `open-optimizer.exe` + processos `msedgewebview2.exe` filhos no Gerenciador de Tarefas → ≤ 60 MB.
  6. `node scripts/bump-version.mjs 0.1.1` → commit → `git tag v0.1.1` → push → após o Release, abrir a v0.1.0 instalada → banner aparece → Atualizar → app reinicia e Configurações mostra `0.1.1`.

- [ ] **Step 11: Fechar** — marcar os critérios de pronto na spec (§12) com os números medidos (tamanho, RAM) e `git commit -am "docs: critérios de pronto do subprojeto 1 medidos"` + push.
