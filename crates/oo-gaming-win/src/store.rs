use oo_gaming::{PersistedSession, SessionStore};
use std::path::PathBuf;

/// Sessão em andamento gravada em disco ANTES de trocar o plano de energia.
pub struct FileStore {
    path: PathBuf,
}

impl FileStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl SessionStore for FileStore {
    fn load(&self) -> Option<PersistedSession> {
        crate::read_json(&self.path)
    }
    fn save(&mut self, s: &PersistedSession) -> Result<(), String> {
        crate::write_json_atomic(&self.path, s).map_err(|e| e.to_string())
    }
    fn clear(&mut self) -> Result<(), String> {
        match std::fs::remove_file(&self.path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    }
}
