use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum GamingState {
    #[default]
    Idle,
    Active,
}

/// Escrito pelo daemon em `gaming-status.json`; lido pela UI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct GamingStatus {
    pub state: GamingState,
    pub game: Option<String>,
    /// RFC 3339 UTC do início da sessão atual.
    pub since: Option<String>,
    pub sessions: u32,
    pub seconds_in_game: u64,
    pub purges: u32,
    pub purged_mb: u64,
    pub daemon_version: String,
}
