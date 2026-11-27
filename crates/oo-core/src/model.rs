use oo_sys::{RegKey, RegValue, SysError};
use serde::{Deserialize, Serialize};

/// Operação primitiva sobre o sistema; sabe capturar e restaurar o estado anterior (via Engine).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    RegSet { key: RegKey, value: RegValue },
    RegDelete { key: RegKey },
    PowerPlan { source_guid: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Before {
    Reg(Option<RegValue>),
    Power { previous: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum After {
    Reg(Option<RegValue>),
    Power { created: Option<String> },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Step {
    pub action: Action,
    pub before: Before,
    pub after: After,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeStatus {
    Pending,
    Applied,
    Reverted,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PostHook {
    RefreshMouse,
}

/// Registro no journal: um tweak aplicado (ou tentado) com os valores originais.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Change {
    pub id: String,
    pub tweak_id: String,
    pub batch_id: String,
    pub timestamp: String,
    pub steps: Vec<Step>,
    pub status: ChangeStatus,
    #[serde(default)]
    pub post: Option<PostHook>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Risk {
    Safe,
    Advanced,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TweakDef {
    pub id: String,
    pub category: String,
    pub risk: Risk,
    pub requires_reboot: bool,
    pub weight: u32,
    pub actions: Vec<Action>,
    pub post: Option<PostHook>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TweakState {
    Applied,
    NotApplied,
    Partial,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, Serialize, Deserialize)]
pub enum OoError {
    #[error("system: {0}")]
    Sys(#[from] SysError),
    #[error("conflict in change {change_id}")]
    Conflict { change_id: String },
    #[error("not found: {0}")]
    NotFound(String),
    #[error("io: {0}")]
    Io(String),
}

impl From<std::io::Error> for OoError {
    fn from(e: std::io::Error) -> Self {
        OoError::Io(e.to_string())
    }
}
