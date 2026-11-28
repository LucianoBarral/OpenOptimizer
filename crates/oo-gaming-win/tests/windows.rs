#![cfg(windows)]
use oo_gaming::GamingSys;
use oo_gaming_win::{task, WinGamingSys};

/// Bin de teste (`tests/support/fake_presentmon.rs`), compilado pelo Cargo antes dos testes.
fn fake_presentmon() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_fake_presentmon"))
}

#[test]
fn fps_capture_le_o_stdout_e_grava_a_sessao() {
    use oo_gaming_win::fps::FpsCapture;
    let c = FpsCapture::start(&fake_presentmon(), "game.exe", 1_000).expect("subiu");
    let rec = c.finish(1_090).expect("houve frames");
    assert_eq!(rec.game, "game.exe");
    assert_eq!((rec.started, rec.duration_s), (1_000, 90));
    // 80 s de frames de 10 ms, menos os 15 s de aquecimento.
    assert!(
        (rec.summary.avg_fps - 100.0).abs() < 0.5,
        "{}",
        rec.summary.avg_fps
    );
    assert!(
        (rec.summary.measured_s - 65.0).abs() < 0.2,
        "{}",
        rec.summary.measured_s
    );
    assert!(rec.valid);
}

#[test]
fn fps_capture_encerra_presentmon_que_nao_sai_sozinho() {
    use oo_gaming_win::fps::FpsCapture;
    let t = std::time::Instant::now();
    let c = FpsCapture::start(&fake_presentmon(), "hang.exe", 0).expect("subiu");
    let rec = c.finish(70);
    assert!(rec.is_some(), "frames lidos antes do kill contam");
    let s = t.elapsed().as_secs();
    assert!(
        (5..20).contains(&s),
        "esperou {s} s (tolerância de 5 s + kill)"
    );
}

#[test]
fn fps_capture_sem_executavel_nao_quebra() {
    use oo_gaming_win::fps::FpsCapture;
    let nada = std::env::temp_dir().join("nao-existe-presentmon.exe");
    assert!(FpsCapture::start(&nada, "game.exe", 0).is_none());
}

fn sys() -> WinGamingSys {
    WinGamingSys::new(std::env::temp_dir().join("oo-gaming-test"))
}

#[test]
fn snapshot_sees_this_test_process_without_handles() {
    let me = std::env::current_exe().unwrap();
    let name = me.file_name().unwrap().to_str().unwrap().to_lowercase();
    assert!(
        sys().running_exes().contains(&name),
        "{name} não apareceu no snapshot"
    );
}

#[test]
fn reads_real_free_memory() {
    let mb = sys().free_mb();
    assert!(mb > 0 && mb < 1_000_000, "free_mb = {mb}");
}

#[test]
fn timer_request_roundtrip() {
    let s = sys();
    s.set_timer(true).unwrap();
    s.set_timer(false).unwrap();
}

/// Criar tarefa com RunLevel=Highest exige admin: no CI (admin) roda; local sem admin, pula.
#[test]
fn scheduled_task_install_query_uninstall() {
    let name = r"OpenOptimizer\_test";
    let exe = std::env::current_exe().unwrap();
    match task::install(name, &exe) {
        Ok(()) => {
            assert!(task::exists(name));
            task::uninstall(name).unwrap();
            assert!(!task::exists(name));
        }
        Err(e) => eprintln!("pulado (provavelmente sem admin): {e}"),
    }
}
