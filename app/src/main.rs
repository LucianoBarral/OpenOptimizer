#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod commands;
mod restore;

use std::path::PathBuf;
use std::sync::Mutex;

fn data_dir() -> PathBuf {
    PathBuf::from(std::env::var("ProgramData").unwrap_or_else(|_| r"C:\ProgramData".into()))
        .join("OpenOptimizer")
}

fn main() {
    let dir = data_dir();
    let logs = dir.join("logs");
    let _ = std::fs::create_dir_all(&logs);
    // panic = "abort" no release: grava o motivo antes de encerrar.
    std::panic::set_hook(Box::new(move |info| {
        let _ = std::fs::write(
            logs.join("panic.log"),
            format!("{info}\n{}", std::backtrace::Backtrace::force_capture()),
        );
    }));
    let journal = oo_core::Journal::open(dir.join("journal.jsonl"))
        .expect("não foi possível abrir o journal");
    let state = commands::AppState {
        engine: Mutex::new(oo_core::Engine::new(
            oo_sys::windows::WindowsBackend::new(),
            journal,
        )),
        catalog: oo_catalog::load_catalog(),
        cpu: Mutex::new(oo_metrics::CpuSampler::new()),
        restore_attempted: Mutex::new(false),
    };
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::hardware,
            commands::metrics,
            commands::tweaks,
            commands::score,
            commands::apply_tweaks,
            commands::revert_tweak,
            commands::revert_change,
            commands::revert_all,
            commands::history,
            commands::pending
        ])
        .run(tauri::generate_context!())
        .expect("falha ao iniciar o Tauri");
}
