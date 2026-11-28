//! Estatísticas de frames a partir do CSV do PresentMon (formato `--v1_metrics`).
//!
//! Puro (sem Windows): o daemon passa as linhas que o PresentMon escreve no stdout, frame a
//! frame; aqui só existem histogramas de tamanho fixo, então a memória não cresce com a sessão.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Write;
use std::path::Path;

/// Segundos iniciais ignorados (carregamento e compilação de shaders derrubam o FPS).
pub const WARMUP_S: f64 = 15.0;
/// Uma sessão só entra em comparações com pelo menos isto de jogo medido.
pub const MIN_VALID_S: f64 = 60.0;

const BUCKET_MS: f64 = 0.05;
const BUCKETS: usize = 5000; // 0–250 ms; acima disso vai para o estouro
const HISTORY_MAX: usize = 600;
const HISTORY_KEEP: usize = 500;

/// Um frame apresentado.
#[derive(Debug, Clone, PartialEq)]
pub struct Frame {
    pub swapchain: String,
    pub runtime: String,
    pub t_s: f64,
    pub ms: f64,
}

#[derive(Debug, Clone, Copy)]
struct Cols {
    swapchain: usize,
    runtime: usize,
    t: usize,
    ms: usize,
}

/// Lê o CSV linha a linha. A primeira linha é o cabeçalho; as colunas são achadas pelo nome.
#[derive(Debug, Default)]
pub struct CsvReader {
    cols: Option<Cols>,
    seen_header: bool,
}

impl CsvReader {
    pub fn new() -> Self {
        Self::default()
    }

    /// Devolve o frame da linha, ou `None` para cabeçalho, linha vazia ou inválida.
    pub fn line(&mut self, line: &str) -> Option<Frame> {
        let line = line.trim_end_matches(['\r', '\n']);
        if line.is_empty() {
            return None;
        }
        if !self.seen_header {
            self.seen_header = true;
            let names: Vec<&str> = line.split(',').map(str::trim).collect();
            let find = |n: &str| names.iter().position(|c| c.eq_ignore_ascii_case(n));
            self.cols = Some(Cols {
                swapchain: find("SwapChainAddress")?,
                runtime: find("Runtime")?,
                t: find("TimeInSeconds")?,
                ms: find("msBetweenPresents")?,
            });
            return None;
        }
        let c = self.cols?;
        let f: Vec<&str> = line.split(',').collect();
        let get = |i: usize| f.get(i).map(|s| s.trim());
        let ms: f64 = get(c.ms)?.parse().ok()?;
        let t_s: f64 = get(c.t)?.parse().ok()?;
        if !ms.is_finite() || ms <= 0.0 || !t_s.is_finite() {
            return None;
        }
        Some(Frame {
            swapchain: get(c.swapchain)?.to_string(),
            runtime: get(c.runtime)?.to_string(),
            t_s,
            ms,
        })
    }
}

#[derive(Debug, Clone)]
struct Hist {
    runtime: String,
    buckets: Vec<u32>,
    over: u32,
    max_ms: f64,
    frames: u64,
    sum_ms: f64,
}

impl Hist {
    fn new(runtime: &str) -> Self {
        Self {
            runtime: runtime.to_string(),
            buckets: vec![0; BUCKETS],
            over: 0,
            max_ms: 0.0,
            frames: 0,
            sum_ms: 0.0,
        }
    }

    fn push(&mut self, ms: f64) {
        let i = (ms / BUCKET_MS) as usize;
        match self.buckets.get_mut(i) {
            Some(b) => *b += 1,
            None => self.over += 1,
        }
        self.max_ms = self.max_ms.max(ms);
        self.frames += 1;
        self.sum_ms += ms;
    }

    /// Frametime no percentil `p` (0–1), pelo centro do bucket.
    fn percentile(&self, p: f64) -> f64 {
        let rank = ((p * self.frames as f64).ceil() as u64).max(1);
        let mut acc = 0u64;
        for (i, &n) in self.buckets.iter().enumerate() {
            acc += n as u64;
            if acc >= rank {
                return (i as f64 + 0.5) * BUCKET_MS;
            }
        }
        self.max_ms
    }
}

/// Acumula os frames de uma sessão (um histograma por swap chain).
#[derive(Debug)]
pub struct FrameStats {
    warmup_s: f64,
    first_t: Option<f64>,
    chains: HashMap<String, Hist>,
}

impl FrameStats {
    pub fn new(warmup_s: f64) -> Self {
        Self {
            warmup_s,
            first_t: None,
            chains: HashMap::new(),
        }
    }

    pub fn push(&mut self, f: &Frame) {
        let first = *self.first_t.get_or_insert(f.t_s);
        if f.t_s < first + self.warmup_s {
            return;
        }
        self.chains
            .entry(f.swapchain.clone())
            .or_insert_with(|| Hist::new(&f.runtime))
            .push(f.ms);
    }

    /// Sumário da swap chain com mais frames (a do jogo; launcher/overlay ficam de fora).
    pub fn summary(&self) -> Option<Summary> {
        let h = self.chains.values().max_by_key(|h| h.frames)?;
        if h.frames == 0 || h.sum_ms <= 0.0 {
            return None;
        }
        let p99 = h.percentile(0.99);
        let p999 = h.percentile(0.999);
        Some(Summary {
            frames: h.frames,
            measured_s: h.sum_ms / 1000.0,
            avg_fps: h.frames as f64 / h.sum_ms * 1000.0,
            low1_fps: 1000.0 / p99,
            low01_fps: 1000.0 / p999,
            ft_p50_ms: h.percentile(0.5),
            ft_p99_ms: p99,
            runtime: h.runtime.clone(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Summary {
    pub frames: u64,
    pub measured_s: f64,
    pub avg_fps: f64,
    pub low1_fps: f64,
    pub low01_fps: f64,
    pub ft_p50_ms: f64,
    pub ft_p99_ms: f64,
    pub runtime: String,
}

/// Uma linha de `fps-history.jsonl`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FpsRecord {
    pub game: String,
    /// Início da sessão (unix, segundos).
    pub started: u64,
    pub duration_s: u64,
    #[serde(flatten)]
    pub summary: Summary,
    /// `measured_s ≥ MIN_VALID_S`: só sessões válidas entram em comparações.
    pub valid: bool,
}

impl FpsRecord {
    pub fn new(game: &str, started: u64, duration_s: u64, summary: Summary) -> Self {
        Self {
            game: game.to_string(),
            started,
            duration_s,
            valid: summary.measured_s >= MIN_VALID_S,
            summary,
        }
    }
}

/// Lê o histórico; linhas corrompidas são ignoradas, arquivo ausente = vazio.
pub fn read_history(path: &Path) -> Vec<FpsRecord> {
    std::fs::read_to_string(path)
        .map(|t| {
            t.lines()
                .filter_map(|l| serde_json::from_str(l).ok())
                .collect()
        })
        .unwrap_or_default()
}

/// Acrescenta um registro. Passando de 600 linhas, reescreve só as 500 mais recentes.
pub fn append_history(path: &Path, rec: &FpsRecord) -> std::io::Result<()> {
    let line = serde_json::to_string(rec).map_err(std::io::Error::other)?;
    {
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        writeln!(f, "{line}")?;
    }
    // Contar linhas é barato; só desserializa quando precisa podar.
    let lines = std::fs::read(path)?.iter().filter(|&&b| b == b'\n').count();
    if lines > HISTORY_MAX {
        let all = read_history(path);
        let keep = &all[all.len() - HISTORY_KEEP..];
        let mut out = String::new();
        for r in keep {
            out.push_str(&serde_json::to_string(r).map_err(std::io::Error::other)?);
            out.push('\n');
        }
        let tmp = path.with_extension("jsonl.tmp");
        std::fs::write(&tmp, out)?;
        std::fs::rename(&tmp, path)?;
    }
    Ok(())
}
