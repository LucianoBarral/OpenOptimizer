use oo_core::{Action, PostHook, Risk, TweakDef};
use oo_sys::{Hive, RegKey, RegValue};
use serde::Deserialize;

#[derive(Deserialize)]
struct File {
    tweak: Vec<Raw>,
}

#[derive(Deserialize)]
struct Raw {
    id: String,
    category: String,
    weight: u32,
    #[serde(default)]
    requires_reboot: bool,
    #[serde(default)]
    advanced: bool,
    #[serde(default)]
    post: Option<String>,
    #[allow(dead_code)]
    rationale: String,
    set: Vec<RawSet>,
}

#[derive(Deserialize)]
struct RawSet {
    hive: String,
    path: String,
    name: String,
    dword: Option<u32>,
    sz: Option<String>,
}

const ULTIMATE_PERFORMANCE: &str = "e9a42b02-d5df-448d-aa00-03f14749eb61";

/// Os `panic!` só disparam com o TOML embutido inválido; o teste abaixo os pega no CI.
pub fn load_catalog() -> Vec<TweakDef> {
    let f: File = toml::from_str(include_str!("../tweaks.toml")).expect("tweaks.toml inválido");
    let mut out: Vec<TweakDef> = f
        .tweak
        .into_iter()
        .map(|r| TweakDef {
            id: r.id,
            category: r.category,
            weight: r.weight,
            requires_reboot: r.requires_reboot,
            risk: if r.advanced {
                Risk::Advanced
            } else {
                Risk::Safe
            },
            post: match r.post.as_deref() {
                None => None,
                Some("refresh_mouse") => Some(PostHook::RefreshMouse),
                Some(o) => panic!("post desconhecido: {o}"),
            },
            actions: r
                .set
                .into_iter()
                .map(|s| {
                    let hive = match s.hive.as_str() {
                        "HKLM" => Hive::Hklm,
                        "HKCU" => Hive::Hkcu,
                        h => panic!("hive desconhecida: {h}"),
                    };
                    let value = match (s.dword, s.sz) {
                        (Some(d), None) => RegValue::Dword(d),
                        (None, Some(z)) => RegValue::Sz(z),
                        _ => panic!("{}: informe exatamente um de dword/sz", s.name),
                    };
                    Action::RegSet {
                        key: RegKey::new(hive, &s.path, &s.name),
                        value,
                    }
                })
                .collect(),
        })
        .collect();
    out.push(TweakDef {
        id: "ultimate-power-plan".into(),
        category: "energy".into(),
        weight: 2,
        requires_reboot: false,
        risk: Risk::Safe,
        post: None,
        actions: vec![Action::PowerPlan {
            source_guid: ULTIMATE_PERFORMANCE.into(),
        }],
    });
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_the_five_starter_tweaks() {
        let c = load_catalog();
        let ids: Vec<_> = c.iter().map(|t| t.id.as_str()).collect();
        assert_eq!(
            ids,
            [
                "power-throttling-off",
                "mmcss-games",
                "game-dvr-off",
                "mouse-accel-off",
                "ultimate-power-plan"
            ]
        );
        assert!(c.iter().all(|t| !t.actions.is_empty() && t.weight > 0));
        assert_eq!(
            c.iter().find(|t| t.id == "mouse-accel-off").unwrap().post,
            Some(PostHook::RefreshMouse)
        );
    }
}
