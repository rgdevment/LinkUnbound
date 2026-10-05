use serde::Serialize;

use linkunbound_core::Language;

use crate::{HERE, catalogue, same_name, shown_origin, store, system, update};

const NOTICES: &str = include_str!("../../../THIRD-PARTY-BUNDLED.md");
const LICENCES: &str = include_str!("../../../THIRD-PARTY-LICENSES.md");

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Build {
    version: &'static str,
    license: &'static str,
    repository: &'static str,
    candidates: bool,
    candidates_apply: bool,
    kept_by_the_store: bool,
}

#[tauri::command]
pub fn about() -> Build {
    let kept = update::looked(store().dir());
    let store_copy = update::route().route == update::Route::Store;
    Build {
        version: HERE,
        license: "GPL-3.0-only",
        repository: "https://github.com/rgdevment/LinkUnbound",
        candidates: update::tracking(HERE, kept.candidates),
        candidates_apply: !store_copy,
        kept_by_the_store: store_copy,
    }
}

#[tauri::command]
pub fn notices(licences: bool) -> &'static str {
    if licences { LICENCES } else { NOTICES }
}

#[tauri::command]
pub fn maintenance_report() -> Result<String, String> {
    let state = system::state();
    let facts = vec![
        ("versión".to_owned(), env!("CARGO_PKG_VERSION").to_owned()),
        ("sistema".to_owned(), std::env::consts::OS.to_owned()),
        ("registrado".to_owned(), state.registered.to_string()),
        ("predeterminado".to_owned(), state.is_default.to_string()),
        ("diagnóstico".to_owned(), format!("{:?}", state.health)),
        (
            "asociaciones".to_owned(),
            format!(
                "{} de {}",
                state.associations.iter().filter(|a| a.held).count(),
                state.associations.len()
            ),
        ),
        (
            "arranca con el sistema".to_owned(),
            state.starts_with_system.to_string(),
        ),
        (
            "Edge instalado".to_owned(),
            state.edge_installed.to_string(),
        ),
        (
            "navegadores".to_owned(),
            catalogue()
                .iter()
                .map(|b| b.name.clone())
                .collect::<Vec<_>>()
                .join(", "),
        ),
    ];
    let store = store();
    let prefs = store.prefs();
    let words = Language::chosen(prefs.locale).strings();
    let mut rules = store.rules().unwrap_or_default();
    for rule in &mut rules.rules {
        // The name a person reads, with the key the rule matches by when they differ.
        rule.source_app = rule
            .source_app
            .as_deref()
            .map(|saved| match shown_origin(saved) {
                name if same_name(&name, saved) => name,
                name => format!("{name} ({saved})"),
            });
    }
    let errors = linkunbound_core::journal(store.dir());
    let body = linkunbound_core::diagnostics(HERE, &facts, &rules, &prefs, &words, &errors);

    let named = format!("{}.md", words.report_file);
    let target = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map_or_else(std::env::temp_dir, std::path::PathBuf::from)
        .join("Desktop")
        .join(&named);
    let target = if target.parent().is_some_and(std::path::Path::is_dir) {
        target
    } else {
        std::env::temp_dir().join(&named)
    };
    std::fs::write(&target, body).map_err(|e| e.to_string())?;
    Ok(target.to_string_lossy().into_owned())
}
