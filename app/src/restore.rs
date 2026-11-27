use std::os::windows::process::CommandExt;

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Cria um Ponto de Restauração do Windows. Best-effort: retorna `false` se o
/// Windows recusar (limite de 1 a cada 24 h, proteção do sistema desligada).
/// O journal próprio continua sendo a garantia principal de reversão.
pub fn create_restore_point() -> bool {
    std::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "Checkpoint-Computer -Description 'Open Optimizer' -RestorePointType MODIFY_SETTINGS -ErrorAction Stop",
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
