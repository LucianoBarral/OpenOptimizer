use oo_core::{engine::new_batch_id, Change, Engine, OoError, TweakDef};
use oo_sys::windows::WindowsBackend;
use oo_views::*;
use std::sync::Mutex;
use tauri::{Emitter, State};

pub struct AppState {
    pub engine: Mutex<Engine<WindowsBackend>>,
    pub catalog: Vec<TweakDef>,
    pub cpu: Mutex<oo_metrics::CpuSampler>,
    pub restore_attempted: Mutex<bool>,
}

#[tauri::command]
pub fn hardware() -> HardwareView {
    let h = oo_metrics::hardware();
    HardwareView {
        cpu: h.cpu,
        gpus: h.gpus,
        ram_bytes: h.ram_bytes as f64,
    }
}

#[tauri::command]
pub fn metrics(s: State<AppState>) -> MetricsView {
    let (used, total) = oo_metrics::memory();
    MetricsView {
        cpu: s.cpu.lock().unwrap().sample(),
        mem_used: used as f64,
        mem_total: total as f64,
        timer_ms: oo_metrics::timer_resolution_ms(),
    }
}

#[tauri::command]
pub fn tweaks(s: State<AppState>) -> Vec<TweakView> {
    let e = s.engine.lock().unwrap();
    s.catalog
        .iter()
        .map(|t| TweakView {
            id: t.id.clone(),
            category: t.category.clone(),
            requires_reboot: t.requires_reboot,
            state: format!("{:?}", e.detect(t)),
        })
        .collect()
}

#[tauri::command]
pub fn score(s: State<AppState>) -> u8 {
    s.engine.lock().unwrap().score(&s.catalog)
}

/// Aplica os tweaks um a um (falha isolada por tweak) e emite `apply-progress`.
#[tauri::command]
pub async fn apply_tweaks(
    app: tauri::AppHandle,
    s: State<'_, AppState>,
    ids: Vec<String>,
) -> Result<BatchResult, ()> {
    let mut restore_point = true;
    {
        let mut tried = s.restore_attempted.lock().unwrap();
        if !*tried {
            restore_point = crate::restore::create_restore_point();
            *tried = true;
        }
    }
    let batch_id = new_batch_id();
    let total = ids.len() as u32;
    let mut results = Vec::with_capacity(ids.len());
    for (i, id) in ids.iter().enumerate() {
        let _ = app.emit(
            "apply-progress",
            Progress {
                index: i as u32 + 1,
                total,
                tweak_id: id.clone(),
            },
        );
        let r = match s.catalog.iter().find(|t| &t.id == id) {
            Some(t) => s.engine.lock().unwrap().apply(t, &batch_id).map(|_| ()),
            None => Err(OoError::NotFound(id.clone())),
        };
        results.push(ApplyResult {
            tweak_id: id.clone(),
            ok: r.is_ok(),
            error: r.err().as_ref().map(ErrorView::from),
        });
    }
    Ok(BatchResult {
        batch_id,
        restore_point,
        results,
    })
}

#[tauri::command]
pub fn revert_tweak(s: State<AppState>, id: String, force: bool) -> Result<(), ErrorView> {
    s.engine
        .lock()
        .unwrap()
        .revert_tweak(&id, force)
        .map_err(|e| (&e).into())
}

#[tauri::command]
pub fn revert_change(s: State<AppState>, id: String, force: bool) -> Result<(), ErrorView> {
    s.engine
        .lock()
        .unwrap()
        .revert(&id, force)
        .map_err(|e| (&e).into())
}

#[tauri::command]
pub fn revert_all(s: State<AppState>, force: bool) -> Vec<ErrorView> {
    s.engine
        .lock()
        .unwrap()
        .revert_all(force)
        .iter()
        .map(|(_, e)| e.into())
        .collect()
}

fn view(c: Change) -> ChangeView {
    ChangeView {
        id: c.id,
        tweak_id: c.tweak_id,
        batch_id: c.batch_id,
        timestamp: c.timestamp,
        status: format!("{:?}", c.status),
    }
}

#[tauri::command]
pub fn history(s: State<AppState>) -> Vec<ChangeView> {
    s.engine
        .lock()
        .unwrap()
        .history()
        .into_iter()
        .rev()
        .map(view)
        .collect()
}

#[tauri::command]
pub fn pending(s: State<AppState>) -> Vec<ChangeView> {
    s.engine
        .lock()
        .unwrap()
        .pending()
        .into_iter()
        .map(view)
        .collect()
}
