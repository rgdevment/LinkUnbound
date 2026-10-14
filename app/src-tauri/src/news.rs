use std::sync::Mutex;

use tauri::Manager;

use crate::{HERE, store};

const NEWS: &str = include_str!("../../src/news.json");

#[derive(Default)]
pub struct Waiting(Mutex<Vec<String>>);

#[derive(serde::Deserialize)]
struct Told {
    version: String,
}

fn told() -> Vec<String> {
    serde_json::from_str::<Vec<Told>>(NEWS)
        .map(|all| all.into_iter().map(|one| one.version).collect())
        .unwrap_or_default()
}

pub fn raise(app: &tauri::AppHandle) {
    let prefs = store().prefs();
    let seen = prefs.news_seen.as_deref();
    // No star mark and no rule is a copy nobody has used yet, whatever version laid it down.
    let fresh =
        prefs.here_since.is_none() && store().rules().map_or(true, |set| set.rules.is_empty());
    let versions = linkunbound_core::untold_news(fresh, seen, HERE, &told());
    if versions.is_empty() && linkunbound_core::news_behind(seen, HERE) {
        mark();
    }
    app.manage(Waiting(Mutex::new(versions)));
}

fn mark() {
    let _ = store().edit_prefs(|prefs| {
        prefs.news_seen = Some(HERE.to_owned());
        true
    });
}

#[tauri::command]
pub fn news(waiting: tauri::State<'_, Waiting>) -> Vec<String> {
    waiting
        .0
        .lock()
        .map(|held| held.clone())
        .unwrap_or_default()
}

#[tauri::command]
pub fn news_seen(waiting: tauri::State<'_, Waiting>) {
    if let Ok(mut held) = waiting.0.lock() {
        held.clear();
    }
    mark();
}

#[cfg(test)]
mod tests {
    use super::told;

    #[test]
    fn every_entry_in_the_news_names_a_version_semver_reads() {
        let all = told();
        assert!(!all.is_empty(), "news.json could not be read");
        for one in all {
            assert!(semver::Version::parse(&one).is_ok(), "{one}");
        }
    }
}
