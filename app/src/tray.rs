//! Ícone na bandeja do sistema (área de notificação) e o ciclo esconder/mostrar.
//! Os textos do menu vêm da UI (`set_tray_labels`), que é quem conhece o idioma.
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, Wry};

pub struct TrayItems {
    pub open: MenuItem<Wry>,
    pub optimize: MenuItem<Wry>,
    pub quit: MenuItem<Wry>,
}

pub fn show_main_window(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        crate::memory::set_low_memory(&w, false);
        let _ = w.unminimize();
        let _ = w.show();
        let _ = w.set_focus();
    }
}

pub fn build(app: &tauri::App) -> tauri::Result<()> {
    let open = MenuItem::with_id(app, "open", "Abrir Open Optimizer", true, None::<&str>)?;
    let optimize = MenuItem::with_id(app, "optimize", "Otimizar agora", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Sair", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &optimize, &quit])?;

    let mut tray = TrayIconBuilder::with_id("main")
        .tooltip("Open Optimizer")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, e| match e.id().as_ref() {
            "open" => show_main_window(app),
            "optimize" => {
                show_main_window(app);
                let _ = app.emit("tray-optimize", ());
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, e| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = e
            {
                show_main_window(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;
    app.manage(TrayItems {
        open,
        optimize,
        quit,
    });
    Ok(())
}

#[tauri::command]
pub fn set_tray_labels(
    items: tauri::State<TrayItems>,
    open: String,
    optimize: String,
    quit: String,
) {
    let _ = items.open.set_text(open);
    let _ = items.optimize.set_text(optimize);
    let _ = items.quit.set_text(quit);
}

/// Esconde a janela (continua na bandeja) e libera a memória do WebView2.
#[tauri::command]
pub fn hide_to_tray(window: tauri::WebviewWindow) {
    crate::memory::set_low_memory(&window, true);
    let _ = window.hide();
}

#[tauri::command]
pub fn show_main(app: AppHandle) {
    show_main_window(&app);
}

#[tauri::command]
pub fn quit_app(app: AppHandle) {
    app.exit(0);
}
