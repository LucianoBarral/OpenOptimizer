//! Ícone grande da janela (barra de tarefas).
//!
//! Causa medida (2026-11-27): o Tauri só define o ícone PEQUENO da janela
//! (WM_GETICON ICON_BIG = 0). Sem o grande, a barra de tarefas busca o ícone no
//! cache de ícones do Explorer — que ainda tinha o asterisco da v0.1.0. Definindo
//! ICON_BIG a partir do ícone embutido no próprio .exe, a barra usa a janela e
//! deixa de depender do cache.
use std::os::windows::ffi::OsStrExt;
use tauri::WebviewWindow;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::Shell::SHDefExtractIconW;
use windows::Win32::UI::WindowsAndMessaging::{SendMessageW, HICON, ICON_BIG, WM_SETICON};

pub fn set_big_icon(win: &WebviewWindow) {
    let (Ok(hwnd), Ok(exe)) = (win.hwnd(), std::env::current_exe()) else {
        return;
    };
    let path: Vec<u16> = exe.as_os_str().encode_wide().chain(Some(0)).collect();
    let mut large = HICON::default();
    unsafe {
        // 48 px: a barra reduz para o tamanho dela com boa qualidade em qualquer escala.
        if SHDefExtractIconW(PCWSTR(path.as_ptr()), 0, 0, Some(&mut large), None, 48).is_ok()
            && !large.is_invalid()
        {
            SendMessageW(
                HWND(hwnd.0 as _),
                WM_SETICON,
                Some(WPARAM(ICON_BIG as usize)),
                Some(LPARAM(large.0 as isize)),
            );
        }
    }
}
