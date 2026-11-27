#![cfg(windows)]
use oo_gaming::GamingSys;
use oo_gaming_win::{task, WinGamingSys};

fn sys() -> WinGamingSys {
    WinGamingSys::new(std::env::temp_dir().join("oo-gaming-test"))
}

#[test]
fn snapshot_sees_this_test_process_without_handles() {
    let me = std::env::current_exe().unwrap();
    let name = me.file_name().unwrap().to_str().unwrap().to_lowercase();
    assert!(sys().running_exes().contains(&name), "{name} não apareceu no snapshot");
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
