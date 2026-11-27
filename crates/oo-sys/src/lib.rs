use serde::{Deserialize, Serialize};

pub mod fake;
pub mod powercfg;
#[cfg(windows)]
pub mod windows;

pub use fake::FakeBackend;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Hive {
    Hklm,
    Hkcu,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RegKey {
    pub hive: Hive,
    pub path: String,
    pub name: String,
}

impl RegKey {
    pub fn new(hive: Hive, path: &str, name: &str) -> Self {
        Self {
            hive,
            path: path.to_string(),
            name: name.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegValue {
    Dword(u32),
    Sz(String),
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, Serialize, Deserialize)]
pub enum SysError {
    #[error("access denied")]
    AccessDenied,
    #[error("not found")]
    NotFound,
    #[error("win32 error {0}")]
    Win32(i32),
    #[error("io: {0}")]
    Io(String),
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
