#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod commands;
mod gaming;
mod memory;
mod restore;
mod tray;

use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{Emitter, Manager};

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
        .plugin(tauri_plugin_dialog::init())
        .manage(state)
        .setup(|app| {
            tray::build(app)?;
            Ok(())
        })
        .on_window_event(|window, event| match event {
            // Minimizado → WebView2 em modo de memória baixa; restaurado → normal.
            tauri::WindowEvent::Resized(_) => {
                let minimized = window.is_minimized().unwrap_or(false);
                if let Some(w) = window.app_handle().get_webview_window(window.label()) {
                    memory::set_low_memory(&w, minimized);
                }
            }
            // O X não fecha direto: a UI decide (bandeja ou sair), conforme a preferência do usuário.
            tauri::WindowEvent::CloseRequested { api, .. } => {
                api.prevent_close();
                let _ = window.emit("close-requested", ());
            }
            _ => {}
        })
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
            commands::pending,
            tray::set_tray_labels,
            tray::hide_to_tray,
            tray::show_main,
            tray::quit_app,
            gaming::gaming_get,
            gaming::gaming_set_enabled,
            gaming::gaming_set_game,
            gaming::gaming_add_game,
            gaming::gaming_scan
        ])
        .run(tauri::generate_context!())
        .expect("falha ao iniciar o Tauri");
}
