use crate::*;
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

#[derive(Default)]
struct State {
    reg: HashMap<RegKey, RegValue>,
    fail_set: HashSet<String>,
    schemes: Vec<String>,
    active: String,
    next: u32,
    mouse_refreshes: u32,
}

/// Backend em memória para testes: nenhum efeito no sistema real.
pub struct FakeBackend(Mutex<State>);

impl FakeBackend {
    pub fn new() -> Self {
        let balanced = "381b4222-f694-41f0-9685-ff5bb260df2e".to_string();
        Self(Mutex::new(State {
            schemes: vec![balanced.clone()],
            active: balanced,
            ..Default::default()
        }))
    }
    /// Faz `reg_set` falhar com AccessDenied para valores com este nome.
    pub fn fail_on_set(&self, name: &str) {
        self.0.lock().unwrap().fail_set.insert(name.to_string());
    }
    pub fn mouse_refreshes(&self) -> u32 {
        self.0.lock().unwrap().mouse_refreshes
    }
}

impl Default for FakeBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemBackend for FakeBackend {
    fn reg_get(&self, key: &RegKey) -> Result<Option<RegValue>, SysError> {
        Ok(self.0.lock().unwrap().reg.get(key).cloned())
    }
    fn reg_set(&self, key: &RegKey, value: &RegValue) -> Result<(), SysError> {
        let mut s = self.0.lock().unwrap();
        if s.fail_set.contains(&key.name) {
            return Err(SysError::AccessDenied);
        }
        s.reg.insert(key.clone(), value.clone());
        Ok(())
    }
    fn reg_delete(&self, key: &RegKey) -> Result<(), SysError> {
        self.0.lock().unwrap().reg.remove(key);
        Ok(())
    }
    fn power_active(&self) -> Result<String, SysError> {
        Ok(self.0.lock().unwrap().active.clone())
    }
    fn power_duplicate(&self, _source: &str) -> Result<String, SysError> {
        let mut s = self.0.lock().unwrap();
        s.next += 1;
        let g = format!("00000000-0000-0000-0000-{:012}", s.next);
        s.schemes.push(g.clone());
        Ok(g)
    }
    fn power_set_active(&self, guid: &str) -> Result<(), SysError> {
        let mut s = self.0.lock().unwrap();
        if !s.schemes.iter().any(|g| g == guid) {
            return Err(SysError::NotFound);
        }
        s.active = guid.to_string();
        Ok(())
    }
    fn power_delete(&self, guid: &str) -> Result<(), SysError> {
        let mut s = self.0.lock().unwrap();
        if s.active == guid {
            return Err(SysError::AccessDenied);
        }
        s.schemes.retain(|g| g != guid);
        Ok(())
    }
    fn refresh_mouse(&self) -> Result<(), SysError> {
        self.0.lock().unwrap().mouse_refreshes += 1;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    fn k() -> RegKey {
        RegKey::new(Hive::Hkcu, r"Software\T", "V")
    }

    #[test]
    fn get_missing_is_none() {
        assert_eq!(FakeBackend::new().reg_get(&k()).unwrap(), None);
    }

    #[test]
    fn set_get_delete_roundtrip() {
        let b = FakeBackend::new();
        b.reg_set(&k(), &RegValue::Dword(7)).unwrap();
        assert_eq!(b.reg_get(&k()).unwrap(), Some(RegValue::Dword(7)));
        b.reg_delete(&k()).unwrap();
        assert_eq!(b.reg_get(&k()).unwrap(), None);
    }

    #[test]
    fn injected_failure() {
        let b = FakeBackend::new();
        b.fail_on_set("V");
        assert!(matches!(
            b.reg_set(&k(), &RegValue::Dword(1)),
            Err(SysError::AccessDenied)
        ));
    }

    #[test]
    fn power_duplicate_and_delete() {
        let b = FakeBackend::new();
        let orig = b.power_active().unwrap();
        let g = b
            .power_duplicate("e9a42b02-d5df-448d-aa00-03f14749eb61")
            .unwrap();
        b.power_set_active(&g).unwrap();
        assert_eq!(b.power_active().unwrap(), g);
        assert!(b.power_delete(&g).is_err(), "não pode apagar o plano ativo");
        b.power_set_active(&orig).unwrap();
        b.power_delete(&g).unwrap();
        assert!(b.power_set_active(&g).is_err());
    }
}
