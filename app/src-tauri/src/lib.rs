mod launch;
mod place;
mod shell;
mod shortcut;
mod system;

use std::sync::Mutex;

use linkunbound_core::{Browser, HostPattern, Rule, Store, Target, host_of, normalise};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

#[derive(Default)]
struct Pending {
    url: Option<String>,
    source_app: Option<String>,
}

#[derive(Clone, Serialize)]
struct Incoming {
    url: String,
    source_app: Option<String>,
}

#[derive(Serialize)]
struct Destinations {
    browsers: Vec<Browser>,
    is_default: bool,
}

fn link_from(args: &[String]) -> Option<String> {
    args.iter().skip(1).find_map(|arg| normalise(arg))
}

/// Where the 1.x line kept its files, so an upgrade finds them in place.
fn store() -> Store {
    let base = std::env::var_os("LOCALAPPDATA")
        .or_else(|| std::env::var_os("APPDATA"))
        .map_or_else(std::env::temp_dir, std::path::PathBuf::from);
    Store::at(base.join("LinkUnbound"))
}

#[cfg(windows)]
fn clicked_in() -> Option<String> {
    linkunbound_win::source_app()
}

#[cfg(not(windows))]
fn clicked_in() -> Option<String> {
    None
}

fn destinations() -> Destinations {
    Destinations {
        browsers: system::browsers(),
        is_default: system::state().is_default,
    }
}

/// The window is created hidden at startup and never destroyed: the click path
/// costs a show, not a webview.
fn deliver(app: &AppHandle, url: String) {
    let source_app = clicked_in();

    // A rule that already answers this link means no window at all: the picker
    // is for the questions nobody has answered yet. A rule that cannot be
    // honoured falls through to the picker rather than opening somewhere else.
    if let Ok(rules) = store().rules()
        && let Some(host) = host_of(&url)
        && let Some(rule) = rules.resolve(&host, source_app.as_deref())
        && launch::open(
            &destinations().browsers,
            &rule.target.browser_id,
            rule.target.profile_id.as_deref(),
            rule.private,
            &url,
        )
        .is_ok()
    {
        return;
    }

    {
        let state = app.state::<Mutex<Pending>>();
        let Ok(mut held) = state.lock() else { return };
        held.url = Some(url.clone());
        held.source_app.clone_from(&source_app);
    }

    let Some(window) = app.get_webview_window("picker") else {
        return;
    };
    let _ = window.emit("link:incoming", Incoming { url, source_app });
    place::at_cursor(&window);
    let _ = window.show();
    let _ = window.set_focus();
}

/// Pulled, not pushed: on a cold start `deliver` runs before the webview exists,
/// so an emitted link would land on nobody.
#[tauri::command]
fn picker_boot(state: tauri::State<'_, Mutex<Pending>>) -> Option<Incoming> {
    let held = state.lock().ok()?;
    Some(Incoming {
        url: held.url.clone()?,
        source_app: held.source_app.clone(),
    })
}

#[tauri::command]
fn picker_destinations() -> Destinations {
    destinations()
}

/// Takes no URL: the webview says which destination was chosen, never what to
/// open. The link comes from state the core already validated.
#[tauri::command]
fn picker_open(
    app: AppHandle,
    state: tauri::State<'_, Mutex<Pending>>,
    browser_id: String,
    profile_id: Option<String>,
    private: bool,
    remember: bool,
) -> Result<(), String> {
    let (url, source_app) = {
        let held = state.lock().map_err(|_| "state is poisoned".to_owned())?;
        (
            held.url
                .clone()
                .ok_or_else(|| "no link pending".to_owned())?,
            held.source_app.clone(),
        )
    };

    launch::open(
        &destinations().browsers,
        &browser_id,
        profile_id.as_deref(),
        private,
        &url,
    )
    .map_err(|e| e.to_string())?;

    if remember && let Some(host) = host_of(&url) {
        let store = store();
        let mut rules = store.rules().map_err(|e| e.to_string())?;
        rules.upsert(Rule {
            id: format!("{host}-{browser_id}"),
            host: HostPattern::Suffix(host),
            source_app,
            target: Target {
                browser_id,
                profile_id,
            },
            private,
        });
        store.save_rules(&rules).map_err(|e| e.to_string())?;
    }

    shell::hide_picker(&app);
    Ok(())
}

#[tauri::command]
fn system_state() -> system::SystemState {
    system::state()
}

#[tauri::command]
fn system_set_registered(enabled: bool) -> Result<system::SystemState, String> {
    system::set_registered(enabled)
}

#[tauri::command]
fn system_set_startup(enabled: bool) -> Result<system::SystemState, String> {
    system::set_starts_with_system(enabled)
}

#[tauri::command]
fn picker_dismiss(app: AppHandle) {
    shell::hide_picker(&app);
}

#[tauri::command]
fn settings_open(app: AppHandle) {
    shell::open_settings(&app);
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            if let Some(url) = link_from(&argv) {
                deliver(app, url);
            }
        }))
        .manage(Mutex::new(Pending::default()))
        .invoke_handler(tauri::generate_handler![
            picker_boot,
            picker_destinations,
            picker_open,
            picker_dismiss,
            settings_open,
            system_state,
            system_set_registered,
            system_set_startup
        ])
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            let args: Vec<String> = std::env::args().collect();
            system::reconcile();

            if args.iter().any(|a| a == "--register") {
                app.handle().exit(0);
                return Ok(());
            }
            shell::install_tray(app.handle())?;
            shortcut::install(app.handle());

            // Launched by the shell for a link: no settings window, just the
            // picker. Launched on its own: the tray, and nothing on screen.
            if let Some(url) = link_from(&args) {
                deliver(app.handle(), url);
            } else if !args.iter().any(|a| a == "--background") {
                shell::open_settings(app.handle());
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("tauri failed to start");
}

#[cfg(test)]
mod tests {
    use super::link_from;

    fn args(rest: &[&str]) -> Vec<String> {
        std::iter::once("linkunbound.exe")
            .chain(rest.iter().copied())
            .map(str::to_owned)
            .collect()
    }

    #[test]
    fn a_teams_link_arrives_already_unwrapped() {
        let raw = "microsoft-edge:https://eu01.safelinks.protection.outlook.com/?url=https%3A%2F%2Fgithub.com%2Fa";
        assert_eq!(
            link_from(&args(&["--background", raw])).as_deref(),
            Some("https://github.com/a")
        );
    }

    #[test]
    fn switches_never_pass_for_a_link() {
        assert!(link_from(&args(&["--register"])).is_none());
        assert!(link_from(&args(&["--gpu-launcher=calc.exe"])).is_none());
    }

    #[cfg(windows)]
    #[test]
    fn the_destinations_offered_are_real_browsers() {
        for browser in super::destinations().browsers {
            assert!(!browser.exe.is_empty());
            assert!(!browser.name.to_lowercase().contains("linkunbound"));
            assert!(!browser.name.to_lowercase().contains("internet explorer"));
        }
    }
}
