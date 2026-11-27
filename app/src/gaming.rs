//! Comandos do Modo Gaming. O app escreve `gaming.json`, cria/remove a tarefa agendada
//! e aplica os tweaks persistentes pelo motor de reversão; quem age em tempo de jogo é o daemon.
use crate::commands::AppState;
use oo_core::{Action, Risk, TweakDef, TweakState};
use oo_gaming::GamingSys;
use oo_gaming::{merge_discovered, GameEntry, GamingConfig, GamingStatus};
use oo_gaming_win::{
    config_path, discovery, read_json, status_path, task, write_json_atomic, WinGamingSys,
};
use oo_sys::{Hive, RegKey, RegValue};
use oo_views::{ErrorView, GameView, GamingStatusView, GamingView};
use tauri::State;

const DAEMON_EXE: &str = "oo-gamingd.exe";

fn io_err(msg: impl std::fmt::Display) -> ErrorView {
    eprintln!("gaming: {msg}");
    ErrorView {
        kind: "Io".into(),
        code: None,
        change_id: None,
    }
}

/// No Windows 11, sem esta chave o timer pedido por um processo sem janela é ignorado.
pub fn global_timer_tweak() -> TweakDef {
    TweakDef {
        id: "gaming-global-timer".into(),
        category: "fps".into(),
        risk: Risk::Safe,
        requires_reboot: false,
        weight: 0,
        post: None,
        actions: vec![Action::RegSet {
            key: RegKey::new(
                Hive::Hklm,
                r"SYSTEM\CurrentControlSet\Control\Session Manager\kernel",
                "GlobalTimerResolutionRequests",
            ),
            value: RegValue::Dword(1),
        }],
    }
}

/// Prioridade alta aplicada pelo próprio Windows ao abrir o jogo (sem tocar no processo).
pub fn ifeo_tweak(exe: &str) -> TweakDef {
    TweakDef {
        id: format!("ifeo:{}", exe.to_lowercase()),
        category: "fps".into(),
        risk: Risk::Safe,
        requires_reboot: false,
        weight: 0,
        post: None,
        actions: vec![Action::RegSet {
            key: RegKey::new(
                Hive::Hklm,
                &format!(
                    r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options\{exe}\PerfOptions"
                ),
                "CpuPriorityClass",
            ),
            value: RegValue::Dword(3),
        }],
    }
}

fn load_config() -> GamingConfig {
    read_json(&config_path()).unwrap_or_default()
}

fn save_config(c: &GamingConfig) -> Result<(), ErrorView> {
    write_json_atomic(&config_path(), c).map_err(io_err)
}

fn daemon_path() -> Result<std::path::PathBuf, ErrorView> {
    Ok(std::env::current_exe()
        .map_err(io_err)?
        .with_file_name(DAEMON_EXE))
}

fn apply_if_needed(s: &AppState, t: &TweakDef) {
    let mut e = s.engine.lock().unwrap();
    if e.detect(t) != TweakState::Applied {
        let _ = e.apply(t, &oo_core::engine::new_batch_id());
    }
}

fn revert_if_applied(s: &AppState, id: &str) {
    let _ = s.engine.lock().unwrap().revert_tweak(id, true);
}

fn view(c: &GamingConfig) -> GamingView {
    let st: GamingStatus = read_json(&status_path()).unwrap_or_default();
    let running = WinGamingSys::new(oo_gaming_win::data_dir())
        .running_exes()
        .contains(DAEMON_EXE);
    GamingView {
        enabled: c.enabled,
        games: c
            .games
            .iter()
            .map(|g| GameView {
                exe: g.exe.clone(),
                name: g.name.clone(),
                source: g.source.clone(),
                enabled: g.enabled,
            })
            .collect(),
        status: GamingStatusView {
            state: format!(
                "{:?}",
                if running {
                    st.state
                } else {
                    oo_gaming::GamingState::Idle
                }
            ),
            game: if running { st.game } else { None },
            since: if running { st.since } else { None },
            sessions: st.sessions,
            seconds_in_game: st.seconds_in_game as f64,
            purges: st.purges,
            purged_mb: st.purged_mb as f64,
        },
        daemon_running: running,
    }
}

#[tauri::command]
pub fn gaming_get() -> GamingView {
    view(&load_config())
}

#[tauri::command]
pub fn gaming_set_enabled(s: State<AppState>, enabled: bool) -> Result<GamingView, ErrorView> {
    let mut c = load_config();
    if enabled {
        if c.games.is_empty() {
            merge_discovered(&mut c.games, discovery::discover_all());
        }
        apply_if_needed(&s, &global_timer_tweak());
        for g in c.games.iter().filter(|g| g.enabled) {
            apply_if_needed(&s, &ifeo_tweak(&g.exe));
        }
        c.enabled = true;
        save_config(&c)?;
        task::install(task::TASK_NAME, &daemon_path()?).map_err(io_err)?;
        task::run(task::TASK_NAME).map_err(io_err)?;
    } else {
        // O daemon lê `enabled=false`, desfaz a sessão e sai sozinho.
        c.enabled = false;
        save_config(&c)?;
        task::uninstall(task::TASK_NAME).map_err(io_err)?;
        revert_if_applied(&s, "gaming-global-timer");
        for g in &c.games {
            revert_if_applied(&s, &ifeo_tweak(&g.exe).id);
        }
    }
    Ok(view(&c))
}

#[tauri::command]
pub fn gaming_set_game(
    s: State<AppState>,
    exe: String,
    enabled: bool,
) -> Result<GamingView, ErrorView> {
    let mut c = load_config();
    if let Some(g) = c
        .games
        .iter_mut()
        .find(|g| g.exe.eq_ignore_ascii_case(&exe))
    {
        g.enabled = enabled;
    }
    save_config(&c)?;
    if enabled && c.enabled {
        apply_if_needed(&s, &ifeo_tweak(&exe));
    } else {
        revert_if_applied(&s, &ifeo_tweak(&exe).id);
    }
    Ok(view(&c))
}

#[tauri::command]
pub fn gaming_add_game(s: State<AppState>, path: String) -> Result<GamingView, ErrorView> {
    let p = std::path::Path::new(&path);
    let exe = p
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| io_err("caminho inválido"))?
        .to_string();
    let name = p
        .file_stem()
        .and_then(|n| n.to_str())
        .unwrap_or(&exe)
        .to_string();
    let mut c = load_config();
    merge_discovered(
        &mut c.games,
        vec![GameEntry {
            exe: exe.clone(),
            name,
            source: "manual".into(),
            enabled: true,
        }],
    );
    save_config(&c)?;
    if c.enabled {
        apply_if_needed(&s, &ifeo_tweak(&exe));
    }
    Ok(view(&c))
}

#[tauri::command]
pub fn gaming_scan(s: State<AppState>) -> Result<GamingView, ErrorView> {
    let mut c = load_config();
    merge_discovered(&mut c.games, discovery::discover_all());
    save_config(&c)?;
    if c.enabled {
        for g in c.games.iter().filter(|g| g.enabled) {
            apply_if_needed(&s, &ifeo_tweak(&g.exe));
        }
    }
    Ok(view(&c))
}
