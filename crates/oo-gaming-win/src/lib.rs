//! Implementação Windows do Modo Gaming. Nunca abre handle em processo de jogo:
//! a detecção usa só o snapshot de nomes do Toolhelp.
#![cfg(windows)]
pub mod discovery;
pub mod store;
pub mod task;
pub mod winsys;

pub use store::FileStore;
pub use winsys::WinGamingSys;

use std::path::PathBuf;

/// `%ProgramData%\OpenOptimizer` (mesmo diretório do journal do app).
/// `OO_DATA_DIR` sobrescreve, só para desenvolvimento e testes.
pub fn data_dir() -> PathBuf {
    if let Ok(d) = std::env::var("OO_DATA_DIR") {
        return PathBuf::from(d);
    }
    PathBuf::from(std::env::var("ProgramData").unwrap_or_else(|_| r"C:\ProgramData".into()))
        .join("OpenOptimizer")
}

pub fn config_path() -> PathBuf {
    data_dir().join("gaming.json")
}

pub fn status_path() -> PathBuf {
    data_dir().join("gaming-status.json")
}

/// Grava JSON de forma atômica (arquivo temporário + rename) para a UI nunca ler meio arquivo.
pub fn write_json_atomic<T: serde::Serialize>(
    path: &std::path::Path,
    value: &T,
) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("tmp");
    std::fs::write(
        &tmp,
        serde_json::to_vec_pretty(value).map_err(std::io::Error::other)?,
    )?;
    std::fs::rename(&tmp, path)
}

pub fn read_json<T: serde::de::DeserializeOwned>(path: &std::path::Path) -> Option<T> {
    serde_json::from_slice(&std::fs::read(path).ok()?).ok()
}
