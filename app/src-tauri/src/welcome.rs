use std::sync::Mutex;

use crate::{shell, store, system};

/// The page the third step opens: any link would do, as long as it goes through the system.
const TRY_ME: &str = "https://rgdevment.com";

/// When the test link was sent, so an older pick does not answer for it.
#[derive(Default)]
pub struct Trying(Mutex<Option<u64>>);

/// Due for anyone who never closed it and is not handing links over yet; someone already set
/// up is marked as welcomed without being shown anything.
pub fn due() -> bool {
    if store().prefs().welcomed {
        return false;
    }
    if system::state().is_default {
        mark();
        return false;
    }
    true
}

fn mark() {
    let _ = store().edit_prefs(|prefs| {
        let was = prefs.welcomed;
        prefs.welcomed = true;
        !was
    });
}

/// Closing it however it closes is having seen it; the window marks itself on its way out.
pub fn on_closed(window: &tauri::Window, event: &tauri::WindowEvent) {
    if window.label() == shell::WELCOME && matches!(event, tauri::WindowEvent::Destroyed) {
        mark();
    }
}

/// Async: a window built from a command on the main thread waits on itself.
#[tauri::command(async)]
pub fn welcome_show(app: tauri::AppHandle) {
    shell::open_welcome(&app, store().prefs().theme);
}

#[tauri::command]
pub fn welcome_close(window: tauri::WebviewWindow) {
    let _ = window.close();
}

#[tauri::command]
pub fn welcome_try(trying: tauri::State<'_, Trying>) -> Result<(), String> {
    if let Ok(mut since) = trying.0.lock() {
        *since = crate::star::now_in_seconds();
    }
    tauri_plugin_opener::open_url(TRY_ME, None::<&str>).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn welcome_tried(trying: tauri::State<'_, Trying>) -> bool {
    let since = trying.0.lock().ok().and_then(|held| *held);
    tried(since, store().picked_at())
}

fn tried(since: Option<u64>, picked: Option<u64>) -> bool {
    matches!((since, picked), (Some(since), Some(picked)) if picked >= since)
}

#[cfg(test)]
mod tests {
    use super::tried;

    #[test]
    fn only_a_pick_after_the_test_link_counts() {
        assert!(tried(Some(100), Some(100)));
        assert!(tried(Some(100), Some(130)));
        assert!(!tried(Some(100), Some(99)), "an older link answers nothing");
        assert!(!tried(Some(100), None), "nothing opened yet");
        assert!(!tried(None, Some(130)), "the test link was never sent");
    }
}
