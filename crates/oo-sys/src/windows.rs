use crate::*;
use std::os::windows::process::CommandExt;
use std::process::Command;
use winreg::enums::*;

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Backend real: registro via winreg, energia via powercfg, mouse via SystemParametersInfo.
pub struct WindowsBackend;

impl WindowsBackend {
    pub fn new() -> Self {
        WindowsBackend
    }
}

impl Default for WindowsBackend {
    fn default() -> Self {
        Self::new()
    }
}

fn root(h: Hive) -> winreg::RegKey {
    winreg::RegKey::predef(match h {
        Hive::Hklm => HKEY_LOCAL_MACHINE,
        Hive::Hkcu => HKEY_CURRENT_USER,
    })
}

fn is_nf(e: &std::io::Error) -> bool {
    e.kind() == std::io::ErrorKind::NotFound
}

fn map(e: std::io::Error) -> SysError {
    match e.kind() {
        std::io::ErrorKind::NotFound => SysError::NotFound,
        std::io::ErrorKind::PermissionDenied => SysError::AccessDenied,
        _ => e
            .raw_os_error()
            .map(SysError::Win32)
            .unwrap_or_else(|| SysError::Io(e.to_string())),
    }
}

/// PowerShell 5.1, relativo ao System32.
pub const POWERSHELL: &str = r"WindowsPowerShell\v1.0\powershell.exe";

/// Caminho absoluto de um executável do System32. O app roda elevado: chamar pelo nome
/// deixaria o Windows procurar na pasta atual e no PATH (binary planting).
/// Sem fallback fixo de propósito: "C:\Windows" pode ser criável por usuário comum
/// quando o Windows está em outra unidade. Se a API falhar, a operação falha.
pub fn system_exe(name: &str) -> Option<std::path::PathBuf> {
    use ::windows::Win32::System::SystemInformation::GetSystemDirectoryW;
    let mut buf = [0u16; 260];
    let n = unsafe { GetSystemDirectoryW(Some(&mut buf)) } as usize;
    (n > 0 && n < buf.len())
        .then(|| std::path::Path::new(&String::from_utf16_lossy(&buf[..n])).join(name))
}

fn powercfg(args: &[&str]) -> Result<String, SysError> {
    let exe = system_exe("powercfg.exe").ok_or(SysError::NotFound)?;
    let o = Command::new(exe)
        .args(args)
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| SysError::Io(e.to_string()))?;
    if !o.status.success() {
        return Err(SysError::Win32(o.status.code().unwrap_or(-1)));
    }
    Ok(String::from_utf8_lossy(&o.stdout).into_owned())
}

impl SystemBackend for WindowsBackend {
    fn reg_get(&self, k: &RegKey) -> Result<Option<RegValue>, SysError> {
        let key = match root(k.hive).open_subkey_with_flags(&k.path, KEY_READ) {
            Ok(x) => x,
            Err(e) if is_nf(&e) => return Ok(None),
            Err(e) => return Err(map(e)),
        };
        let raw = match key.get_raw_value(&k.name) {
            Ok(r) => r,
            Err(e) if is_nf(&e) => return Ok(None),
            Err(e) => return Err(map(e)),
        };
        Ok(Some(match raw.vtype {
            REG_DWORD if raw.bytes.len() >= 4 => RegValue::Dword(u32::from_le_bytes([
                raw.bytes[0],
                raw.bytes[1],
                raw.bytes[2],
                raw.bytes[3],
            ])),
            REG_SZ | REG_EXPAND_SZ => {
                let wide: Vec<u16> = raw
                    .bytes
                    .as_chunks::<2>()
                    .0
                    .iter()
                    .map(|c| u16::from_le_bytes(*c))
                    .collect();
                RegValue::Sz(
                    String::from_utf16_lossy(&wide)
                        .trim_end_matches('\0')
                        .to_string(),
                )
            }
            _ => {
                return Err(SysError::Io(format!(
                    "tipo de registro não suportado em {}",
                    k.name
                )))
            }
        }))
    }

    fn reg_set(&self, k: &RegKey, v: &RegValue) -> Result<(), SysError> {
        let (key, _) = root(k.hive).create_subkey(&k.path).map_err(map)?;
        match v {
            RegValue::Dword(d) => key.set_value(&k.name, d),
            RegValue::Sz(s) => key.set_value(&k.name, s),
        }
        .map_err(map)
    }

    fn reg_delete(&self, k: &RegKey) -> Result<(), SysError> {
        match root(k.hive).open_subkey_with_flags(&k.path, KEY_SET_VALUE) {
            Ok(key) => match key.delete_value(&k.name) {
                Ok(()) => Ok(()),
                Err(e) if is_nf(&e) => Ok(()),
                Err(e) => Err(map(e)),
            },
            Err(e) if is_nf(&e) => Ok(()),
            Err(e) => Err(map(e)),
        }
    }

    fn power_active(&self) -> Result<String, SysError> {
        powercfg::parse_guid(&powercfg(&["/getactivescheme"])?).ok_or(SysError::NotFound)
    }

    fn power_duplicate(&self, src: &str) -> Result<String, SysError> {
        powercfg::parse_guid(&powercfg(&["/duplicatescheme", src])?).ok_or(SysError::NotFound)
    }

    fn power_set_active(&self, g: &str) -> Result<(), SysError> {
        powercfg(&["/setactive", g]).map(|_| ())
    }

    fn power_delete(&self, g: &str) -> Result<(), SysError> {
        powercfg(&["/delete", g]).map(|_| ())
    }

    /// Aplica os valores atuais de HKCU\Control Panel\Mouse sem precisar de logoff.
    fn refresh_mouse(&self) -> Result<(), SysError> {
        use ::windows::Win32::UI::WindowsAndMessaging::{
            SystemParametersInfoW, SPIF_SENDCHANGE, SPIF_UPDATEINIFILE, SPI_SETMOUSE,
        };
        let read = |n: &str| -> i32 {
            match self.reg_get(&RegKey::new(Hive::Hkcu, r"Control Panel\Mouse", n)) {
                Ok(Some(RegValue::Sz(s))) => s.trim().parse().unwrap_or(0),
                _ => 0,
            }
        };
        let mut v = [
            read("MouseThreshold1"),
            read("MouseThreshold2"),
            read("MouseSpeed"),
        ];
        unsafe {
            SystemParametersInfoW(
                SPI_SETMOUSE,
                0,
                Some(v.as_mut_ptr().cast()),
                SPIF_UPDATEINIFILE | SPIF_SENDCHANGE,
            )
            .map_err(|e| SysError::Win32(e.code().0))
        }
    }
}
