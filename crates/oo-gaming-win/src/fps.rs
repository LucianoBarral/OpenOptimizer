//! Captura de FPS de uma sessão via PresentMon (ETW). O PresentMon recebe só o NOME do exe:
//! nada aqui abre handle, injeta ou lê memória do jogo.

use oo_fps::{CsvReader, FpsRecord, FrameStats, WARMUP_S};
use std::io::{BufRead, BufReader};
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

const CREATE_NO_WINDOW: u32 = 0x0800_0000;
const SESSION: &str = "OpenOptimizerFps";
/// Quanto esperar o PresentMon sair sozinho (ele sai quando o jogo fecha).
const EXIT_GRACE: Duration = Duration::from_secs(5);

/// `presentmon.exe` instalado ao lado do daemon (externalBin do Tauri).
pub fn presentmon_path() -> Option<PathBuf> {
    let p = std::env::current_exe()
        .ok()?
        .parent()?
        .join("presentmon.exe");
    p.is_file().then_some(p)
}

pub fn fps_history_path() -> PathBuf {
    crate::data_dir().join("fps-history.jsonl")
}

pub struct FpsCapture {
    exe: PathBuf,
    child: Child,
    reader: Option<JoinHandle<()>>,
    stats: Arc<Mutex<FrameStats>>,
    game: String,
    started: u64,
}

impl FpsCapture {
    /// Inicia a captura do `game` (nome do exe). `None` se o PresentMon não puder subir:
    /// a sessão do Modo Gaming segue normalmente, só sem medição.
    pub fn start(presentmon: &Path, game: &str, started: u64) -> Option<Self> {
        let mut child = Command::new(presentmon)
            .args([
                "--process_name",
                game,
                "--output_stdout",
                "--v1_metrics",
                "--no_console_stats",
                "--no_track_input",
                "--no_track_gpu",
                "--session_name",
                SESSION,
                "--stop_existing_session",
                "--terminate_on_proc_exit",
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .ok()?;
        let stdout = child.stdout.take()?;
        let stats = Arc::new(Mutex::new(FrameStats::new(WARMUP_S)));
        let sink = Arc::clone(&stats);
        let reader = std::thread::spawn(move || {
            let mut csv = CsvReader::new();
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                if let Some(f) = csv.line(&line) {
                    if let Ok(mut s) = sink.lock() {
                        s.push(&f);
                    }
                }
            }
        });
        Some(Self {
            exe: presentmon.to_path_buf(),
            child,
            reader: Some(reader),
            stats,
            game: game.to_string(),
            started,
        })
    }

    /// Encerra a captura e devolve o registro da sessão (`None` se nenhum frame foi medido).
    pub fn finish(mut self, now: u64) -> Option<FpsRecord> {
        let deadline = Instant::now() + EXIT_GRACE;
        let exited = loop {
            match self.child.try_wait() {
                Ok(Some(_)) => break true,
                Ok(None) if Instant::now() < deadline => {
                    std::thread::sleep(Duration::from_millis(100))
                }
                _ => break false,
            }
        };
        if !exited {
            let _ = self.child.kill();
            let _ = self.child.wait();
            // Morto à força, a sessão de ETW fica aberta: pede ao PresentMon para fechá-la.
            stop_session(&self.exe);
        }
        if let Some(r) = self.reader.take() {
            let _ = r.join();
        }
        let summary = self.stats.lock().ok()?.summary()?;
        Some(FpsRecord::new(
            &self.game,
            self.started,
            now.saturating_sub(self.started),
            summary,
        ))
    }
}

fn stop_session(presentmon: &Path) {
    let child = Command::new(presentmon)
        .args(["--session_name", SESSION, "--terminate_existing_session"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .creation_flags(CREATE_NO_WINDOW)
        .spawn();
    if let Ok(mut c) = child {
        let deadline = Instant::now() + EXIT_GRACE;
        while Instant::now() < deadline {
            if let Ok(Some(_)) = c.try_wait() {
                return;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        let _ = c.kill();
        let _ = c.wait();
    }
}
