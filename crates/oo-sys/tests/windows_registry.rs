#![cfg(windows)]
use oo_sys::{windows::WindowsBackend, Hive, RegKey, RegValue, SystemBackend};

#[test]
fn hkcu_set_get_delete() {
    let b = WindowsBackend::new();
    let k = RegKey::new(Hive::Hkcu, r"Software\OpenOptimizer\_test", "Probe");
    b.reg_set(&k, &RegValue::Dword(42)).unwrap();
    assert_eq!(b.reg_get(&k).unwrap(), Some(RegValue::Dword(42)));
    b.reg_set(&k, &RegValue::Sz("x".into())).unwrap();
    assert_eq!(b.reg_get(&k).unwrap(), Some(RegValue::Sz("x".into())));
    b.reg_delete(&k).unwrap();
    assert_eq!(b.reg_get(&k).unwrap(), None);
    b.reg_delete(&k).unwrap(); // idempotente
    let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
    let _ = hkcu.delete_subkey_all(r"Software\OpenOptimizer\_test");
    // Só apaga o pai se estiver vazio (delete_subkey falha se houver subchaves): sem rastro.
    let _ = hkcu.delete_subkey(r"Software\OpenOptimizer");
}

#[test]
fn power_active_returns_guid() {
    assert_eq!(WindowsBackend::new().power_active().unwrap().len(), 36);
}
