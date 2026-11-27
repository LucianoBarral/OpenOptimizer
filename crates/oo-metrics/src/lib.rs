use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Hardware {
    pub cpu: String,
    pub gpus: Vec<String>,
    pub ram_bytes: u64,
}

#[cfg(windows)]
mod imp {
    use super::Hardware;
    use windows::Win32::Foundation::FILETIME;
    use windows::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
    use windows::Win32::System::Threading::GetSystemTimes;
    use winreg::{enums::HKEY_LOCAL_MACHINE, RegKey};

    fn ft(f: FILETIME) -> u64 {
        ((f.dwHighDateTime as u64) << 32) | f.dwLowDateTime as u64
    }

    /// (idle, kernel+user). O tempo de kernel já inclui o idle.
    fn times() -> (u64, u64) {
        let (mut i, mut k, mut u) = (
            FILETIME::default(),
            FILETIME::default(),
            FILETIME::default(),
        );
        unsafe {
            let _ = GetSystemTimes(Some(&mut i), Some(&mut k), Some(&mut u));
        }
        (ft(i), ft(k) + ft(u))
    }

    /// Uso de CPU por delta entre chamadas (mesma fonte do Gerenciador de Tarefas).
    pub struct CpuSampler {
        last: (u64, u64),
    }

    impl Default for CpuSampler {
        fn default() -> Self {
            Self::new()
        }
    }

    impl CpuSampler {
        pub fn new() -> Self {
            Self { last: times() }
        }

        pub fn sample(&mut self) -> f32 {
            let now = times();
            let (di, dt) = (
                now.0.saturating_sub(self.last.0),
                now.1.saturating_sub(self.last.1),
            );
            self.last = now;
            if dt == 0 {
                0.0
            } else {
                (100.0 * (1.0 - di as f64 / dt as f64)).clamp(0.0, 100.0) as f32
            }
        }
    }

    /// (bytes em uso, bytes totais) de memória física.
    pub fn memory() -> (u64, u64) {
        let mut m = MEMORYSTATUSEX {
            dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32,
            ..Default::default()
        };
        unsafe {
            let _ = GlobalMemoryStatusEx(&mut m);
        }
        (m.ullTotalPhys - m.ullAvailPhys, m.ullTotalPhys)
    }

    #[link(name = "ntdll")]
    extern "system" {
        fn NtQueryTimerResolution(min: *mut u32, max: *mut u32, cur: *mut u32) -> i32;
    }

    /// Resolução atual do timer do sistema em ms (padrão do Windows ≈ 15,6 ms).
    pub fn timer_resolution_ms() -> f64 {
        let (mut a, mut b, mut c) = (0u32, 0u32, 0u32);
        unsafe {
            NtQueryTimerResolution(&mut a, &mut b, &mut c);
        }
        c as f64 / 10_000.0 // unidades de 100 ns
    }

    pub fn hardware() -> Hardware {
        let lm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let cpu = lm
            .open_subkey(r"HARDWARE\DESCRIPTION\System\CentralProcessor\0")
            .and_then(|k| k.get_value::<String, _>("ProcessorNameString"))
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        let mut gpus = vec![];
        if let Ok(cls) = lm.open_subkey(
            r"SYSTEM\CurrentControlSet\Control\Class\{4d36e968-e325-11ce-bfc1-08002be10318}",
        ) {
            for name in cls
                .enum_keys()
                .flatten()
                .filter(|n| n.chars().all(|c| c.is_ascii_digit()))
            {
                let Ok(k) = cls.open_subkey(&name) else {
                    continue;
                };
                // GPU física é sempre PCI; adaptadores virtuais (Parsec, spacedesk, IDDs) não são.
                let pci = k
                    .get_value::<String, _>("MatchingDeviceId")
                    .is_ok_and(|id| id.to_ascii_lowercase().starts_with(r"pci\"));
                if let Ok(d) = k.get_value::<String, _>("DriverDesc") {
                    if pci && !gpus.contains(&d) {
                        gpus.push(d);
                    }
                }
            }
        }
        Hardware {
            cpu,
            gpus,
            ram_bytes: memory().1,
        }
    }
}

#[cfg(windows)]
pub use imp::*;

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    #[test]
    fn readings_are_plausible() {
        let mut c = CpuSampler::new();
        std::thread::sleep(std::time::Duration::from_millis(200));
        assert!((0.0..=100.0).contains(&c.sample()));
        let (used, total) = memory();
        assert!(total > 1 << 30 && used <= total);
        let t = timer_resolution_ms();
        assert!(t > 0.0 && t <= 16.0, "timer {t}");
        let h = hardware();
        assert!(!h.cpu.is_empty());
        assert_eq!(h.ram_bytes, total);
        assert!(
            h.gpus.iter().all(|g| !g.to_lowercase().contains("virtual")),
            "{:?}",
            h.gpus
        );
        println!(
            "cpu={} gpus={:?} ram={}GB timer={t}ms",
            h.cpu,
            h.gpus,
            total >> 30
        );
    }
}
