use oo_gaming::GamingSys;
use oo_sys::{windows::WindowsBackend, SystemBackend};
use std::collections::HashSet;
use std::ffi::c_void;
use std::path::PathBuf;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{CloseHandle, HANDLE, LUID};
use windows::Win32::Security::{
    AdjustTokenPrivileges, LookupPrivilegeValueW, LUID_AND_ATTRIBUTES, SE_PRIVILEGE_ENABLED,
    TOKEN_ADJUST_PRIVILEGES, TOKEN_PRIVILEGES, TOKEN_QUERY,
};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

const ULTIMATE_PERFORMANCE: &str = "e9a42b02-d5df-448d-aa00-03f14749eb61";
const SYSTEM_MEMORY_LIST_INFORMATION: u32 = 0x50;
const MEMORY_PURGE_STANDBY_LIST: i32 = 4;
const PAGE: u64 = 4096;

#[link(name = "ntdll")]
extern "system" {
    fn NtSetTimerResolution(desired: u32, set: u8, current: *mut u32) -> i32;
    fn NtQuerySystemInformation(class: u32, info: *mut c_void, len: u32, ret: *mut u32) -> i32;
    fn NtSetSystemInformation(class: u32, info: *mut c_void, len: u32) -> i32;
}

/// SYSTEM_MEMORY_LIST_INFORMATION (x64).
#[repr(C)]
#[derive(Default)]
struct MemoryList {
    zero: usize,
    free: usize,
    modified: usize,
    modified_no_write: usize,
    bad: usize,
    by_priority: [usize; 8],
    repurposed_by_priority: [usize; 8],
    modified_page_file: usize,
}

pub struct WinGamingSys {
    backend: WindowsBackend,
    ultimate_file: PathBuf,
}

impl WinGamingSys {
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            backend: WindowsBackend::new(),
            ultimate_file: data_dir.join("gaming-ultimate.txt"),
        }
    }
}

fn enable_privilege(name: &str) -> Result<(), String> {
    unsafe {
        let mut token = HANDLE::default();
        // Handle do PRÓPRIO token (processo do daemon), não de outro processo.
        OpenProcessToken(
            GetCurrentProcess(),
            TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY,
            &mut token,
        )
        .map_err(|e| e.to_string())?;
        let wide: Vec<u16> = name.encode_utf16().chain(Some(0)).collect();
        let mut luid = LUID::default();
        let r = LookupPrivilegeValueW(PCWSTR::null(), PCWSTR(wide.as_ptr()), &mut luid);
        let result = r.and_then(|_| {
            let tp = TOKEN_PRIVILEGES {
                PrivilegeCount: 1,
                Privileges: [LUID_AND_ATTRIBUTES {
                    Luid: luid,
                    Attributes: SE_PRIVILEGE_ENABLED,
                }],
            };
            AdjustTokenPrivileges(token, false, Some(&tp), 0, None, None)
        });
        let _ = CloseHandle(token);
        result.map_err(|e| e.to_string())
    }
}

impl GamingSys for WinGamingSys {
    fn running_exes(&self) -> HashSet<String> {
        let mut out = HashSet::new();
        unsafe {
            // Snapshot do sistema: devolve só nomes; nenhum handle de processo é aberto.
            let Ok(snap) = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) else {
                return out;
            };
            let mut e = PROCESSENTRY32W {
                dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
                ..Default::default()
            };
            if Process32FirstW(snap, &mut e).is_ok() {
                loop {
                    let len = e
                        .szExeFile
                        .iter()
                        .position(|&c| c == 0)
                        .unwrap_or(e.szExeFile.len());
                    out.insert(String::from_utf16_lossy(&e.szExeFile[..len]).to_lowercase());
                    if Process32NextW(snap, &mut e).is_err() {
                        break;
                    }
                }
            }
            let _ = CloseHandle(snap);
        }
        out
    }

    fn set_timer(&self, on: bool) -> Result<(), String> {
        let mut cur = 0u32;
        let st = unsafe { NtSetTimerResolution(5000, on as u8, &mut cur) }; // 5000 × 100 ns = 0,5 ms
        if st >= 0 {
            Ok(())
        } else {
            Err(format!("NtSetTimerResolution 0x{st:08x}"))
        }
    }

    fn active_scheme(&self) -> Result<String, String> {
        self.backend.power_active().map_err(|e| e.to_string())
    }

    fn set_scheme(&self, guid: &str) -> Result<(), String> {
        self.backend
            .power_set_active(guid)
            .map_err(|e| e.to_string())
    }

    /// Reaproveita o plano que já criamos (GUID guardado) se ele ainda existir; senão duplica o oculto.
    fn ensure_ultimate_scheme(&self) -> Result<String, String> {
        let schemes = self.backend.power_schemes().map_err(|e| e.to_string())?;
        if let Ok(saved) = std::fs::read_to_string(&self.ultimate_file) {
            let saved = saved.trim().to_ascii_lowercase();
            if schemes.contains(&saved) {
                return Ok(saved);
            }
        }
        let g = self
            .backend
            .power_duplicate(ULTIMATE_PERFORMANCE)
            .map_err(|e| e.to_string())?;
        if let Some(dir) = self.ultimate_file.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        std::fs::write(&self.ultimate_file, &g).map_err(|e| e.to_string())?;
        Ok(g)
    }

    fn free_mb(&self) -> u64 {
        let mut m = MemoryList::default();
        let st = unsafe {
            NtQuerySystemInformation(
                SYSTEM_MEMORY_LIST_INFORMATION,
                &mut m as *mut _ as *mut c_void,
                std::mem::size_of::<MemoryList>() as u32,
                std::ptr::null_mut(),
            )
        };
        if st < 0 {
            return u64::MAX; // sem leitura: nunca dispara limpeza por engano
        }
        (m.zero as u64 + m.free as u64) * PAGE / (1024 * 1024)
    }

    fn purge_standby(&self) -> Result<u64, String> {
        enable_privilege("SeProfileSingleProcessPrivilege")?;
        let before = self.free_mb();
        let mut cmd = MEMORY_PURGE_STANDBY_LIST;
        let st = unsafe {
            NtSetSystemInformation(
                SYSTEM_MEMORY_LIST_INFORMATION,
                &mut cmd as *mut _ as *mut c_void,
                4,
            )
        };
        if st < 0 {
            return Err(format!("NtSetSystemInformation 0x{st:08x}"));
        }
        Ok(self.free_mb().saturating_sub(before))
    }
}
