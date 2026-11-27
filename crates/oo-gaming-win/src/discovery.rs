//! Descoberta de jogos instalados — só leitura de registro e arquivos de launcher.
use oo_gaming::GameEntry;
use std::path::{Path, PathBuf};

/// Executáveis que nunca são "o jogo" (instaladores, crash reporters, anti-cheat, helpers).
const NOT_GAME: &[&str] = &[
    "unins",
    "setup",
    "install",
    "crash",
    "report",
    "redist",
    "vcredist",
    "dxsetup",
    "directx",
    "easyanticheat",
    "battleye",
    "be_service",
    "launcher",
    "helper",
    "updater",
    "cefprocess",
    "webhelper",
    "prereq",
    "dotnet",
    "touchup",
];

/// Valores de uma chave em texto VDF/ACF da Steam (`"chave"  "valor"`).
pub fn vdf_values(text: &str, key: &str) -> Vec<String> {
    let needle = format!("\"{key}\"");
    text.lines()
        .filter_map(|l| {
            let l = l.trim();
            let rest = l.strip_prefix(&needle)?.trim();
            let v = rest.strip_prefix('"')?.strip_suffix('"')?;
            Some(v.replace(r"\\", r"\"))
        })
        .collect()
}

/// `(nome, installdir)` de um `appmanifest_*.acf`.
pub fn parse_acf(text: &str) -> Option<(String, String)> {
    Some((
        vdf_values(text, "name").into_iter().next()?,
        vdf_values(text, "installdir").into_iter().next()?,
    ))
}

/// `(DisplayName, nome do exe)` de um manifesto `.item` da Epic.
pub fn parse_epic_item(text: &str) -> Option<(String, String)> {
    let v: serde_json::Value = serde_json::from_str(text).ok()?;
    let name = v.get("DisplayName")?.as_str()?.to_string();
    let launch = v.get("LaunchExecutable")?.as_str()?;
    let exe = Path::new(launch).file_name()?.to_str()?.to_string();
    exe.to_ascii_lowercase()
        .ends_with(".exe")
        .then_some((name, exe))
}

/// Maior `.exe` até 3 níveis abaixo de `dir`, ignorando os que não são o jogo.
pub fn main_exe(dir: &Path) -> Option<String> {
    fn walk(d: &Path, depth: u8, best: &mut Option<(u64, String)>) {
        let Ok(rd) = std::fs::read_dir(d) else { return };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                if depth < 3 {
                    walk(&p, depth + 1, best);
                }
                continue;
            }
            let Some(name) = p.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            let lower = name.to_ascii_lowercase();
            if !lower.ends_with(".exe") || NOT_GAME.iter().any(|b| lower.contains(b)) {
                continue;
            }
            let size = e.metadata().map(|m| m.len()).unwrap_or(0);
            if best.as_ref().is_none_or(|(s, _)| size > *s) {
                *best = Some((size, name.to_string()));
            }
        }
    }
    let mut best = None;
    walk(dir, 0, &mut best);
    best.map(|(_, n)| n)
}

fn pretty_name(exe: &str) -> String {
    let stem = exe
        .strip_suffix(".exe")
        .or_else(|| exe.strip_suffix(".EXE"))
        .unwrap_or(exe);
    stem.replace("-Win64-Shipping", "").replace("_", " ")
}

fn entry(exe: String, name: String, source: &str) -> GameEntry {
    GameEntry {
        exe,
        name,
        source: source.into(),
        enabled: true,
    }
}

/// Jogos que o próprio Windows (Game Bar) já identificou neste PC.
pub fn from_game_config_store() -> Vec<GameEntry> {
    use winreg::{enums::HKEY_CURRENT_USER, RegKey};
    let Ok(children) =
        RegKey::predef(HKEY_CURRENT_USER).open_subkey(r"System\GameConfigStore\Children")
    else {
        return vec![];
    };
    children
        .enum_keys()
        .flatten()
        .filter_map(|k| {
            children
                .open_subkey(&k)
                .ok()?
                .get_value::<String, _>("MatchedExeFullPath")
                .ok()
        })
        .filter_map(|full| {
            let exe = Path::new(&full).file_name()?.to_str()?.to_string();
            Some(entry(exe.clone(), pretty_name(&exe), "windows"))
        })
        .collect()
}

pub fn from_epic() -> Vec<GameEntry> {
    let dir = PathBuf::from(std::env::var("ProgramData").unwrap_or_default())
        .join(r"Epic\EpicGamesLauncher\Data\Manifests");
    let Ok(rd) = std::fs::read_dir(dir) else {
        return vec![];
    };
    rd.flatten()
        .filter(|e| e.path().extension().is_some_and(|x| x == "item"))
        .filter_map(|e| parse_epic_item(&std::fs::read_to_string(e.path()).ok()?))
        .map(|(name, exe)| entry(exe, name, "epic"))
        .collect()
}

pub fn from_steam() -> Vec<GameEntry> {
    use winreg::{enums::HKEY_CURRENT_USER, RegKey};
    let Ok(root) = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey(r"Software\Valve\Steam")
        .and_then(|k| k.get_value::<String, _>("SteamPath"))
    else {
        return vec![];
    };
    let root = PathBuf::from(root);
    let mut libs = vec![root.clone()];
    if let Ok(t) = std::fs::read_to_string(root.join(r"steamapps\libraryfolders.vdf")) {
        libs.extend(vdf_values(&t, "path").into_iter().map(PathBuf::from));
    }
    libs.sort();
    libs.dedup();
    let mut out = vec![];
    for lib in libs {
        let Ok(rd) = std::fs::read_dir(lib.join("steamapps")) else {
            continue;
        };
        for e in rd.flatten() {
            let p = e.path();
            let is_manifest = p
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("appmanifest_"));
            if !is_manifest {
                continue;
            }
            let Some((name, dir)) = std::fs::read_to_string(&p).ok().and_then(|t| parse_acf(&t))
            else {
                continue;
            };
            if name.contains("Redistributable")
                || name.contains("Proton")
                || name.contains("Steam Linux")
            {
                continue;
            }
            if let Some(exe) = main_exe(&lib.join(r"steamapps\common").join(dir)) {
                out.push(entry(exe, name, "steam"));
            }
        }
    }
    out
}

/// Todas as fontes, sem duplicatas (mesmo exe = mesmo jogo; a 1ª fonte vence).
pub fn discover_all() -> Vec<GameEntry> {
    let mut all: Vec<GameEntry> = vec![];
    oo_gaming::merge_discovered(&mut all, from_epic());
    oo_gaming::merge_discovered(&mut all, from_steam());
    oo_gaming::merge_discovered(&mut all, from_game_config_store());
    all
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_steam_library_paths_and_manifest() {
        let lib = "\"libraryfolders\"\n{\n\t\"0\"\n\t{\n\t\t\"path\"\t\t\"C:\\\\Program Files (x86)\\\\Steam\"\n\t}\n\t\"1\"\n\t{\n\t\t\"path\"\t\t\"D:\\\\SteamLibrary\"\n\t}\n}";
        assert_eq!(
            vdf_values(lib, "path"),
            vec![r"C:\Program Files (x86)\Steam", r"D:\SteamLibrary"]
        );
        let acf = "\"AppState\"\n{\n\t\"appid\"\t\t\"730\"\n\t\"name\"\t\t\"Counter-Strike 2\"\n\t\"installdir\"\t\t\"Counter-Strike Global Offensive\"\n}";
        assert_eq!(
            parse_acf(acf),
            Some((
                "Counter-Strike 2".into(),
                "Counter-Strike Global Offensive".into()
            ))
        );
    }

    #[test]
    fn parses_epic_item() {
        let item = r#"{"DisplayName":"Fortnite","InstallLocation":"C:\\Games\\Fortnite","LaunchExecutable":"FortniteGame/Binaries/Win64/FortniteLauncher.exe"}"#;
        assert_eq!(
            parse_epic_item(item),
            Some(("Fortnite".into(), "FortniteLauncher.exe".into()))
        );
        assert_eq!(
            parse_epic_item(r#"{"DisplayName":"X","LaunchExecutable":""}"#),
            None
        );
    }

    #[test]
    fn main_exe_picks_largest_real_game_binary() {
        let d = std::env::temp_dir().join(format!("oo-disc-{}", std::process::id()));
        std::fs::create_dir_all(d.join("bin/win64")).unwrap();
        std::fs::write(d.join("unins000.exe"), vec![0u8; 9000]).unwrap();
        std::fs::write(d.join("bin/win64/game.exe"), vec![0u8; 5000]).unwrap();
        std::fs::write(d.join("bin/CrashReporter.exe"), vec![0u8; 8000]).unwrap();
        std::fs::write(d.join("small.exe"), vec![0u8; 100]).unwrap();
        assert_eq!(main_exe(&d).as_deref(), Some("game.exe"));
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn pretty_names() {
        assert_eq!(pretty_name("VALORANT-Win64-Shipping.exe"), "VALORANT");
        assert_eq!(pretty_name("Rocket_League.exe"), "Rocket League");
    }
}
