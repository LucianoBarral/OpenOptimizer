use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameEntry {
    /// Nome do executável (ex.: "VALORANT-Win64-Shipping.exe").
    pub exe: String,
    pub name: String,
    /// "windows" | "steam" | "epic" | "manual"
    pub source: String,
    pub enabled: bool,
}

fn yes() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct GamingConfig {
    pub enabled: bool,
    #[serde(default = "yes")]
    pub notify: bool,
    #[serde(default)]
    pub games: Vec<GameEntry>,
}

impl GamingConfig {
    /// Primeiro jogo habilitado que está rodando (`running` já em minúsculas).
    pub fn find_enabled(&self, running: &HashSet<String>) -> Option<&GameEntry> {
        self.games
            .iter()
            .find(|g| g.enabled && running.contains(&g.exe.to_lowercase()))
    }
}

/// Adiciona jogos descobertos que ainda não estão na lista (comparação sem
/// diferenciar maiúsculas). Preserva o liga/desliga escolhido pelo usuário.
pub fn merge_discovered(existing: &mut Vec<GameEntry>, found: Vec<GameEntry>) -> usize {
    let mut added = 0;
    for g in found {
        if !existing.iter().any(|e| e.exe.eq_ignore_ascii_case(&g.exe)) {
            existing.push(g);
            added += 1;
        }
    }
    added
}

#[cfg(test)]
mod tests {
    use super::*;

    fn g(exe: &str, enabled: bool) -> GameEntry {
        GameEntry {
            exe: exe.into(),
            name: exe.into(),
            source: "manual".into(),
            enabled,
        }
    }

    #[test]
    fn find_enabled_ignores_case_and_disabled() {
        let c = GamingConfig {
            enabled: true,
            notify: true,
            games: vec![g("Off.exe", false), g("Game.EXE", true)],
        };
        let running: HashSet<String> = ["off.exe".into(), "game.exe".into()].into();
        assert_eq!(c.find_enabled(&running).unwrap().exe, "Game.EXE");
    }

    #[test]
    fn merge_keeps_user_choice_and_skips_duplicates() {
        let mut list = vec![g("a.exe", false)];
        let n = merge_discovered(&mut list, vec![g("A.EXE", true), g("b.exe", true)]);
        assert_eq!(n, 1);
        assert!(!list[0].enabled);
        assert_eq!(list[1].exe, "b.exe");
    }
}
