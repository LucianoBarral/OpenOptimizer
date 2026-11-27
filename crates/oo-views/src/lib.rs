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
