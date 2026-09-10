mod launch;
mod place;
mod shell;
mod shortcut;
mod system;

use std::sync::Mutex;

use linkunbound_core::{Browser, Rule, Scope, Store, Target, host_of, normalise, site_of};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

const PICKER_WIDTH: f64 = 368.0;
/// Below this the webview measured before it painted.
const MIN_HEIGHT: f64 = 80.0;

#[derive(Default)]
struct Pending {
    url: Option<String>,
    source_app: Option<String>,
}

#[derive(Clone, Serialize)]
struct Incoming {
    url: String,
    source_app: Option<String>,
    host: String,
    site: String,
}

impl Incoming {
    fn new(url: String, source_app: Option<String>) -> Self {
        let host = host_of(&url).unwrap_or_default();
        let site = site_of(&host);
        Self {
            url,
            source_app,
            host,
            site,
        }
    }
}

/// `Once` writes nothing, which is why it is the state every link starts in.
#[derive(Clone, Copy, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum Remember {
    Once,
    Url,
    Host,
    Site,
}

impl Remember {
    fn scope(self, url: &str, host: &str) -> Option<Scope> {
        match self {
            Self::Once => None,
            Self::Url => Some(Scope::Url(url.to_owned())),
            Self::Host => Some(Scope::Host(host.to_owned())),
            Self::Site => Some(Scope::Site(site_of(host))),
        }
    }
}

#[derive(Serialize)]
struct Destinations {
    browsers: Vec<Listed>,
    is_default: bool,
}

/// The icon rides along as a data URI: a handful of 2 kB PNGs costs less than
/// opening the asset protocol and scoping it.
#[derive(Serialize)]
struct Listed {
    #[serde(flatten)]
    browser: Browser,
    icon: Option<String>,
}

fn icons_dir() -> std::path::PathBuf {
    std::env::var_os("LOCALAPPDATA")
        .map_or_else(std::env::temp_dir, std::path::PathBuf::from)
        .join("LinkUnbound")
        .join("icons")
}

#[cfg(windows)]
fn icon_data(exe: &str, id: &str) -> Option<String> {
    use base64::Engine;
    let path = linkunbound_win::icon_for(exe, id, &icons_dir())?;
    let bytes = std::fs::read(path).ok()?;
    Some(format!(
        "data:image/png;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(bytes)
    ))
}

#[cfg(not(windows))]
fn icon_data(_exe: &str, _id: &str) -> Option<String> {
    None
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
        browsers: system::browsers()
            .into_iter()
            .map(|browser| Listed {
                icon: icon_data(&browser.exe, &browser.id),
                browser,
            })
            .collect(),
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
        && let Some(rule) = rules.resolve(&url, &host, source_app.as_deref())
        && launch::open(
            &system::browsers(),
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
    let _ = window.emit("link:incoming", Incoming::new(url, source_app));
}

/// Pulled, not pushed: on a cold start `deliver` runs before the webview exists,
/// so an emitted link would land on nobody.
#[tauri::command]
fn picker_boot(state: tauri::State<'_, Mutex<Pending>>) -> Option<Incoming> {
    let held = state.lock().ok()?;
    Some(Incoming::new(held.url.clone()?, held.source_app.clone()))
}

/// Sizing before showing is what keeps the picker from appearing at the wrong size.
#[tauri::command]
fn picker_fit(app: AppHandle, height: f64) {
    let Some(window) = app.get_webview_window("picker") else {
        return;
    };
    let width = window.inner_size().map_or(PICKER_WIDTH, |s| {
        let scale = window.scale_factor().unwrap_or(1.0);
        f64::from(s.width) / scale
    });
    let _ = window.set_size(tauri::LogicalSize::new(width, height.max(MIN_HEIGHT)));
    place::at_cursor(&window);
    let _ = window.show();
    let _ = window.set_focus();
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
    remember: Remember,
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
        &system::browsers(),
        &browser_id,
        profile_id.as_deref(),
        private,
        &url,
    )
    .map_err(|e| e.to_string())?;

    if let Some(host) = host_of(&url)
        && let Some(scope) = remember.scope(&url, &host)
    {
        let store = store();
        let mut rules = store.rules().map_err(|e| e.to_string())?;
        rules.upsert(Rule {
            id: format!("{host}-{browser_id}-{}", rules.rules.len()),
            scope,
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
            picker_fit,
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
    use super::{Remember, link_from};
    use linkunbound_core::Scope;

    const LINK: &str = "https://docs.google.com/document/d/1a9F/edit";

    #[test]
    fn opening_once_writes_no_rule_at_all() {
        assert!(Remember::Once.scope(LINK, "docs.google.com").is_none());
    }

    #[test]
    fn each_scope_saves_the_reach_its_label_promises() {
        assert_eq!(
            Remember::Url.scope(LINK, "docs.google.com"),
            Some(Scope::Url(LINK.to_owned()))
        );
        assert_eq!(
            Remember::Host.scope(LINK, "docs.google.com"),
            Some(Scope::Host("docs.google.com".to_owned()))
        );
        assert_eq!(
            Remember::Site.scope(LINK, "docs.google.com"),
            Some(Scope::Site("google.com".to_owned()))
        );
    }

    /// The whole site of a compound suffix is the registrable domain, never the
    /// suffix: `co.uk` as a rule would answer for every British domain.
    #[test]
    fn the_whole_site_never_reaches_past_the_registrable_domain() {
        assert_eq!(
            Remember::Site.scope("https://www.bbc.co.uk/news", "www.bbc.co.uk"),
            Some(Scope::Site("bbc.co.uk".to_owned()))
        );
    }

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

    /// 1.x moved links through a named pipe with a 4096-byte buffer and a real
    /// Teams link overran it, arriving as truncated JSON that was thrown away.
    #[test]
    fn a_link_far_longer_than_four_kilobytes_survives() {
        let padding = "a".repeat(6000);
        let long = format!(
            "https://teams.microsoft.com/l/message/19:meeting@thread.v2?context={padding}&ce=prod"
        );
        assert!(long.len() > 6000);
        assert_eq!(link_from(&args(&[&long])).as_deref(), Some(long.as_str()));
    }

    #[test]
    fn a_long_link_still_unwraps_out_of_its_wrapper() {
        let padding = "b".repeat(5000);
        let inner = format!("https%3A%2F%2Fgithub.com%2F{padding}");
        let wrapped = format!("https://eu01.safelinks.protection.outlook.com/?url={inner}&data=05");
        let out = link_from(&args(&[&wrapped])).expect("should unwrap");
        assert!(out.starts_with("https://github.com/"));
        assert!(out.len() > 5000);
    }

    #[test]
    fn switches_never_pass_for_a_link() {
        assert!(link_from(&args(&["--register"])).is_none());
        assert!(link_from(&args(&["--gpu-launcher=calc.exe"])).is_none());
    }

    #[cfg(windows)]
    #[test]
    fn the_destinations_offered_are_real_browsers() {
        for listed in super::destinations().browsers {
            let browser = listed.browser;
            assert!(!browser.exe.is_empty());
            assert!(!browser.name.to_lowercase().contains("linkunbound"));
            assert!(!browser.name.to_lowercase().contains("internet explorer"));
        }
    }
}
