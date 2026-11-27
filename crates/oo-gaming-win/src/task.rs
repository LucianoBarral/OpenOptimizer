//! Tarefa Agendada que inicia o daemon no logon, elevada e sem UAC.
//! Criada por XML: o padrão do `schtasks /SC ONLOGON` não inicia na bateria e
//! encerra a tarefa após 72 h.
use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::Command;

pub const TASK_NAME: &str = r"OpenOptimizer\GamingMode";
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

pub fn task_xml(exe: &Path, user: &str) -> String {
    let exe = xml_escape(&exe.display().to_string());
    let user = xml_escape(user);
    format!(
        r#"<?xml version="1.0" encoding="UTF-16"?>
<Task version="1.4" xmlns="http://schemas.microsoft.com/windows/2004/02/mit/task">
  <RegistrationInfo><Description>Open Optimizer - Modo Gaming</Description></RegistrationInfo>
  <Triggers><LogonTrigger><Enabled>true</Enabled><UserId>{user}</UserId></LogonTrigger></Triggers>
  <Principals><Principal id="Author"><UserId>{user}</UserId><LogonType>InteractiveToken</LogonType><RunLevel>HighestAvailable</RunLevel></Principal></Principals>
  <Settings>
    <MultipleInstancesPolicy>IgnoreNew</MultipleInstancesPolicy>
    <DisallowStartIfOnBatteries>false</DisallowStartIfOnBatteries>
    <StopIfGoingOnBatteries>false</StopIfGoingOnBatteries>
    <AllowHardTerminate>true</AllowHardTerminate>
    <StartWhenAvailable>true</StartWhenAvailable>
    <IdleSettings><StopOnIdleEnd>false</StopOnIdleEnd><RestartOnIdle>false</RestartOnIdle></IdleSettings>
    <AllowStartOnDemand>true</AllowStartOnDemand>
    <Enabled>true</Enabled>
    <ExecutionTimeLimit>PT0S</ExecutionTimeLimit>
    <Priority>7</Priority>
  </Settings>
  <Actions Context="Author"><Exec><Command>{exe}</Command></Exec></Actions>
</Task>
"#
    )
}

fn schtasks(args: &[&str]) -> Result<(), String> {
    let exe = oo_sys::windows::system_exe("schtasks.exe").ok_or("System32 indisponível")?;
    let o = Command::new(exe)
        .args(args)
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| e.to_string())?;
    if o.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&o.stderr).trim().to_string())
    }
}

fn current_user() -> String {
    let domain = std::env::var("USERDOMAIN").unwrap_or_default();
    let user = std::env::var("USERNAME").unwrap_or_default();
    if domain.is_empty() {
        user
    } else {
        format!(r"{domain}\{user}")
    }
}

/// Cria (ou substitui) a tarefa `name` apontando para `exe`.
pub fn install(name: &str, exe: &Path) -> Result<(), String> {
    let xml = task_xml(exe, &current_user());
    let path = crate::data_dir().join("gaming-task.xml");
    std::fs::create_dir_all(crate::data_dir()).map_err(|e| e.to_string())?;
    // schtasks espera UTF-16 LE com BOM, como declarado no XML.
    let mut bytes = vec![0xFF, 0xFE];
    for u in xml.encode_utf16() {
        bytes.extend_from_slice(&u.to_le_bytes());
    }
    std::fs::write(&path, bytes).map_err(|e| e.to_string())?;
    let r = schtasks(&[
        "/Create",
        "/TN",
        name,
        "/XML",
        &path.display().to_string(),
        "/F",
    ]);
    let _ = std::fs::remove_file(&path);
    r
}

pub fn run(name: &str) -> Result<(), String> {
    schtasks(&["/Run", "/TN", name])
}

pub fn exists(name: &str) -> bool {
    schtasks(&["/Query", "/TN", name]).is_ok()
}

pub fn uninstall(name: &str) -> Result<(), String> {
    if !exists(name) {
        return Ok(());
    }
    schtasks(&["/Delete", "/TN", name, "/F"])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xml_has_battery_and_time_limit_overrides_and_escapes_path() {
        let x = task_xml(
            Path::new(r"C:\Program Files\A&B\oo-gamingd.exe"),
            r"PC\joão",
        );
        assert!(x.contains("<DisallowStartIfOnBatteries>false</DisallowStartIfOnBatteries>"));
        assert!(x.contains("<StopIfGoingOnBatteries>false</StopIfGoingOnBatteries>"));
        assert!(x.contains("<ExecutionTimeLimit>PT0S</ExecutionTimeLimit>"));
        assert!(x.contains("<RunLevel>HighestAvailable</RunLevel>"));
        assert!(x.contains(r"A&amp;B\oo-gamingd.exe"));
        assert!(x.contains(r"PC\joão"));
    }
}
