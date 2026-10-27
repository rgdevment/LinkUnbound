use linkunbound_core::Asking;

use crate::store;

pub fn now_in_seconds() -> Option<u64> {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .map(|since| since.as_secs())
}

#[tauri::command]
pub fn star_due() -> Result<bool, String> {
    let Some(now) = now_in_seconds() else {
        return Ok(false);
    };
    let store = store();
    let mut due = false;
    store
        .edit_prefs(|prefs| {
            let counted = || store.rules().map(|set| set.rules.len()).unwrap_or_default();
            match linkunbound_core::asking(prefs.asked_for_a_star, prefs.here_since, now, counted) {
                Asking::Start => {
                    prefs.here_since = Some(now);
                    true
                }
                Asking::Wait => false,
                Asking::Now => {
                    due = true;
                    false
                }
            }
        })
        .map_err(|e| e.to_string())?;
    Ok(due)
}

#[tauri::command]
pub fn star_done() -> Result<(), String> {
    store()
        .edit_prefs(|prefs| {
            prefs.asked_for_a_star = true;
            true
        })
        .map(|_| ())
        .map_err(|e| e.to_string())
}
