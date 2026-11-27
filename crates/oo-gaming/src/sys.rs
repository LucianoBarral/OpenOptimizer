use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashSet;

/// O que o Modo Gaming precisa do sistema. A implementação real fica em `oo-gaming-win`.
pub trait GamingSys {
    /// Nomes dos executáveis rodando, em minúsculas (sem abrir handle em processos).
    fn running_exes(&self) -> HashSet<String>;
    fn set_timer(&self, on: bool) -> Result<(), String>;
    fn active_scheme(&self) -> Result<String, String>;
    fn set_scheme(&self, guid: &str) -> Result<(), String>;
    /// GUID do plano Desempenho Máximo (cria se não existir).
    fn ensure_ultimate_scheme(&self) -> Result<String, String>;
    /// Memória realmente livre (Free + Zero), em MB.
    fn free_mb(&self) -> u64;
    /// Limpa a standby list; devolve os MB liberados.
    fn purge_standby(&self) -> Result<u64, String>;
}

/// Estado gravado ANTES de trocar o plano de energia (write-ahead).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistedSession {
    pub game: String,
    pub previous_scheme: Option<String>,
    pub started: u64,
}

pub trait SessionStore {
    fn load(&self) -> Option<PersistedSession>;
    fn save(&mut self, s: &PersistedSession) -> Result<(), String>;
    fn clear(&mut self) -> Result<(), String>;
}

#[derive(Default)]
pub struct MemStore(pub Option<PersistedSession>);

impl SessionStore for MemStore {
    fn load(&self) -> Option<PersistedSession> {
        self.0.clone()
    }
    fn save(&mut self, s: &PersistedSession) -> Result<(), String> {
        self.0 = Some(s.clone());
        Ok(())
    }
    fn clear(&mut self) -> Result<(), String> {
        self.0 = None;
        Ok(())
    }
}

pub const FAKE_BALANCED: &str = "381b4222-f694-41f0-9685-ff5bb260df2e";
pub const FAKE_ULTIMATE: &str = "00000000-0000-0000-0000-0000000000aa";

/// Sistema falso para testes: tudo em memória.
pub struct FakeGamingSys {
    pub running: RefCell<HashSet<String>>,
    pub timer_on: RefCell<bool>,
    pub scheme: RefCell<String>,
    pub free: RefCell<u64>,
    pub purges: RefCell<u32>,
}

impl Default for FakeGamingSys {
    fn default() -> Self {
        Self {
            running: RefCell::new(HashSet::new()),
            timer_on: RefCell::new(false),
            scheme: RefCell::new(FAKE_BALANCED.into()),
            free: RefCell::new(8000),
            purges: RefCell::new(0),
        }
    }
}

impl FakeGamingSys {
    pub fn run(&self, exe: &str) {
        self.running.borrow_mut().insert(exe.to_lowercase());
    }
    pub fn kill(&self, exe: &str) {
        self.running.borrow_mut().remove(&exe.to_lowercase());
    }
}

impl GamingSys for FakeGamingSys {
    fn running_exes(&self) -> HashSet<String> {
        self.running.borrow().clone()
    }
    fn set_timer(&self, on: bool) -> Result<(), String> {
        *self.timer_on.borrow_mut() = on;
        Ok(())
    }
    fn active_scheme(&self) -> Result<String, String> {
        Ok(self.scheme.borrow().clone())
    }
    fn set_scheme(&self, guid: &str) -> Result<(), String> {
        *self.scheme.borrow_mut() = guid.into();
        Ok(())
    }
    fn ensure_ultimate_scheme(&self) -> Result<String, String> {
        Ok(FAKE_ULTIMATE.into())
    }
    fn free_mb(&self) -> u64 {
        *self.free.borrow()
    }
    fn purge_standby(&self) -> Result<u64, String> {
        *self.purges.borrow_mut() += 1;
        let before = *self.free.borrow();
        *self.free.borrow_mut() = before + 3000;
        Ok(3000)
    }
}
