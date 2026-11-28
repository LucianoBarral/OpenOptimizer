//! Structs enviadas para a UI. `ts-rs` gera os tipos TypeScript em
//! `ui/src/lib/ipc/bindings/` ao rodar `cargo test -p open-optimizer`.
use serde::Serialize;
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../../ui/src/lib/ipc/bindings/")]
pub struct HardwareView {
    pub cpu: String,
    pub gpus: Vec<String>,
    pub ram_bytes: f64,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../../ui/src/lib/ipc/bindings/")]
pub struct MetricsView {
    pub cpu: f32,
    pub mem_used: f64,
    pub mem_total: f64,
    pub timer_ms: f64,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../../ui/src/lib/ipc/bindings/")]
pub struct TweakView {
    pub id: String,
    pub category: String,
    pub requires_reboot: bool,
    pub state: String,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../../ui/src/lib/ipc/bindings/")]
pub struct ChangeView {
    pub id: String,
    pub tweak_id: String,
    pub batch_id: String,
    pub timestamp: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../../ui/src/lib/ipc/bindings/")]
pub struct ErrorView {
    pub kind: String,
    pub code: Option<i32>,
    pub change_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../../ui/src/lib/ipc/bindings/")]
pub struct ApplyResult {
    pub tweak_id: String,
    pub ok: bool,
    pub error: Option<ErrorView>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../../ui/src/lib/ipc/bindings/")]
pub struct BatchResult {
    pub batch_id: String,
    pub restore_point: bool,
    pub results: Vec<ApplyResult>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../../ui/src/lib/ipc/bindings/")]
pub struct Progress {
    pub index: u32,
    pub total: u32,
    pub tweak_id: String,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../../ui/src/lib/ipc/bindings/")]
pub struct GameView {
    pub exe: String,
    pub name: String,
    pub source: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../../ui/src/lib/ipc/bindings/")]
pub struct GamingStatusView {
    /// "Idle" | "Active"
    pub state: String,
    pub game: Option<String>,
    pub since: Option<String>,
    pub sessions: u32,
    pub seconds_in_game: f64,
    pub purges: u32,
    pub purged_mb: f64,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../../ui/src/lib/ipc/bindings/")]
pub struct GamingView {
    pub enabled: bool,
    pub games: Vec<GameView>,
    pub status: GamingStatusView,
    pub daemon_running: bool,
}

/// Uma sessão medida (linha de `fps-history.jsonl`).
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../../ui/src/lib/ipc/bindings/")]
pub struct FpsRecordView {
    pub game: String,
    /// Início da sessão (unix, segundos).
    pub started: f64,
    pub duration_s: f64,
    pub measured_s: f64,
    pub frames: f64,
    pub avg_fps: f64,
    pub low1_fps: f64,
    pub low01_fps: f64,
    pub ft_p50_ms: f64,
    pub ft_p99_ms: f64,
    pub runtime: String,
    pub valid: bool,
}

impl From<&oo_fps::FpsRecord> for FpsRecordView {
    fn from(r: &oo_fps::FpsRecord) -> Self {
        let s = &r.summary;
        Self {
            game: r.game.clone(),
            started: r.started as f64,
            duration_s: r.duration_s as f64,
            measured_s: s.measured_s,
            frames: s.frames as f64,
            avg_fps: s.avg_fps,
            low1_fps: s.low1_fps,
            low01_fps: s.low01_fps,
            ft_p50_ms: s.ft_p50_ms,
            ft_p99_ms: s.ft_p99_ms,
            runtime: s.runtime.clone(),
            valid: r.valid,
        }
    }
}

impl From<&oo_core::OoError> for ErrorView {
    fn from(e: &oo_core::OoError) -> Self {
        use oo_core::OoError::*;
        use oo_sys::SysError as S;
        let (kind, code, change_id) = match e {
            Sys(S::AccessDenied) => ("AccessDenied", None, None),
            Sys(S::NotFound) | NotFound(_) => ("NotSupported", None, None),
            Sys(S::Win32(c)) => ("Win32", Some(*c), None),
            Sys(S::Io(_)) | Io(_) => ("Io", None, None),
            Conflict { change_id } => ("Conflict", None, Some(change_id.clone())),
        };
        ErrorView {
            kind: kind.into(),
            code,
            change_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conflict_maps_to_ui_error_with_change_id() {
        let v = ErrorView::from(&oo_core::OoError::Conflict {
            change_id: "c1".into(),
        });
        assert_eq!(v.kind, "Conflict");
        assert_eq!(v.change_id.as_deref(), Some("c1"));
        let w = ErrorView::from(&oo_core::OoError::Sys(oo_sys::SysError::Win32(5)));
        assert_eq!((w.kind.as_str(), w.code), ("Win32", Some(5)));
    }
}
