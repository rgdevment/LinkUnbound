use serde::Serialize;

use crate::{HERE, store, update};

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
