//! Memória do WebView2 quando a janela não está em uso.
//!
//! Medido (2026-11-27): com a janela aberta o app+WebView2 fica em ~108 MB
//! (o processo Rust é só ~7 MB). Minimizado, o usuário provavelmente está
//! jogando: pedimos ao WebView2 o nível de memória `Low`, que descarta caches
//! e reduz o working set, e voltamos a `Normal` ao restaurar.
use tauri::WebviewWindow;

pub fn set_low_memory(win: &WebviewWindow, low: bool) {
    let _ = win.with_webview(move |wv| {
        use webview2_com::Microsoft::Web::WebView2::Win32::{
            ICoreWebView2_19, COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_LOW,
            COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_NORMAL,
        };
        use windows::core::Interface;
        let level = if low {
            COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_LOW
        } else {
            COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_NORMAL
        };
        // SAFETY: chamadas COM na thread da UI (with_webview já executa nela).
        unsafe {
            if let Ok(core) = wv.controller().CoreWebView2() {
                // ICoreWebView2_19 existe no runtime WebView2 ≥ 114; em versões antigas só ignora.
                if let Ok(c19) = core.cast::<ICoreWebView2_19>() {
                    let _ = c19.SetMemoryUsageTargetLevel(level);
                }
            }
        }
    });
}
