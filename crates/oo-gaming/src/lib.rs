//! Lógica do Modo Gaming, independente de Windows (testável com `FakeGamingSys`).
pub mod config;
pub mod session;
pub mod status;
pub mod sys;

pub use config::{merge_discovered, GameEntry, GamingConfig};
pub use session::Session;
pub use status::{GamingState, GamingStatus};
pub use sys::{FakeGamingSys, GamingSys, MemStore, PersistedSession, SessionStore};

/// Limite de memória livre (Free + Zero) abaixo do qual a standby list é limpa.
pub const PURGE_BELOW_MB: u64 = 1024;
/// Intervalo mínimo entre limpezas.
pub const PURGE_MIN_INTERVAL_S: u64 = 300;
