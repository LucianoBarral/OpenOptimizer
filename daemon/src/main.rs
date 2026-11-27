//! oo-gamingd — Modo Gaming do Open Optimizer.
//! Sem janela. Lê `gaming.json`, observa os processos a cada 3 s (snapshot por nome,
//! sem abrir handle em jogos), aplica/desfaz as otimizações e escreve `gaming-status.json`.
//! Sai sozinho quando o Modo Gaming é desligado.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(windows)]
fn main() {
    daemon::run();
}

#[cfg(not(windows))]
fn main() {}

#[cfg(windows)]
mod daemon {
    use oo_gaming::{GamingConfig, GamingState, GamingStatus, Session};
    use oo_gaming_win::{
        config_path, data_dir, read_json, status_path, write_json_atomic, FileStore, WinGamingSys,
    };
    use std::time::{Duration, SystemTime, UNIX_EPOCH};
    use windows::core::w;
    use windows::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS};
    use windows::Win32::System::Threading::CreateMutexW;

    const TICK: Duration = Duration::from_secs(3);
    const ACTIVE_STATUS_EVERY_S: u64 = 30;

    fn now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }

    /// Mantém um mutex nomeado pelo tempo de vida do processo: uma instância só.
    fn single_instance() -> bool {
        unsafe {
            match CreateMutexW(None, true, w!("Global\\OpenOptimizerGamingMode")) {
                Ok(h) => {
                    let first = GetLastError() != ERROR_ALREADY_EXISTS;
                    // HANDLE não fecha no Drop: o mutex vive até o processo sair.
                    let _ = h;
                    first
                }
                Err(_) => false,
            }
        }
    }

    fn mtime(p: &std::path::Path) -> Option<SystemTime> {
        std::fs::metadata(p).and_then(|m| m.modified()).ok()
    }

    pub fn run() {
        if !single_instance() {
            return;
        }
        let dir = data_dir();
        let sys = WinGamingSys::new(dir.clone());
        let mut store = FileStore::new(dir.join("gaming-session.json"));
        Session::recover(&sys, &mut store);

        let mut status: GamingStatus = read_json(&status_path()).unwrap_or_default();
        status.state = GamingState::Idle;
        status.game = None;
        status.since = None;
        status.daemon_version = env!("CARGO_PKG_VERSION").into();
        let _ = write_json_atomic(&status_path(), &status);

        let cfg_path = config_path();
        let mut cfg: GamingConfig = read_json(&cfg_path).unwrap_or_default();
        let mut cfg_time = mtime(&cfg_path);
        let mut session = Session::default();
        let mut last_write = now();

        loop {
            std::thread::sleep(TICK);
            let t = mtime(&cfg_path);
            if t != cfg_time {
                cfg_time = t;
                cfg = read_json(&cfg_path).unwrap_or_default();
            }
            let n = now();
            if !cfg.enabled {
                session.shutdown(&sys, &mut store, n, &mut status);
                let _ = write_json_atomic(&status_path(), &status);
                return;
            }
            let changed = session.tick(&cfg, &sys, &mut store, n, &mut status);
            let periodic = status.state == GamingState::Active
                && n.saturating_sub(last_write) >= ACTIVE_STATUS_EVERY_S;
            if changed || periodic {
                let _ = write_json_atomic(&status_path(), &status);
                last_write = n;
            }
        }
    }
}
