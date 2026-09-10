mod launch;
mod place;
mod shell;
mod shortcut;
mod system;

use std::sync::Mutex;

use linkunbound_core::{Browser, Rule, Scope, Store, Target, host_of, merge, normalise, site_of};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

const PICKER_WIDTH: f64 = 368.0;
/// Below this the webview measured before it painted.
const MIN_HEIGHT: f64 = 80.0;

#[derive(Default)]
struct Pending {
    url: Option<String>,
    source_app: Option<String>,
    /// Echoed back on pick, so a newer link cannot open in place of the one seen.
    token: u64,
}

#[derive(Clone, Serialize)]
struct Incoming {
    url: String,
    source_app: Option<String>,
    host: String,
    site: String,
    token: u64,
}

impl Incoming {
    fn new(url: String, source_app: Option<String>, token: u64) -> Self {
        let host = host_of(&url).unwrap_or_default();
        let site = site_of(&host);
        Self {
            url,
            source_app,
            host,
            site,
            token,
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

#[cfg(windows)]
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

/// The whole tail first: `tauri-plugin-single-instance` joins argv with `|` and
/// splits on it again, tearing apart any link that carries one.
fn link_from(args: &[String]) -> Option<String> {
    let tail = args[1..].join("|");
    normalise(&tail).or_else(|| args.iter().skip(1).find_map(|arg| normalise(arg)))
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

/// Detection plus what the user saved, which is the only place the two meet.
fn catalogue() -> Vec<Browser> {
    let saved = store().browsers().map(|c| c.browsers).unwrap_or_default();
    merge(system::browsers(), &saved)
}

fn destinations() -> Destinations {
    Destinations {
        browsers: catalogue()
            .into_iter()
            .filter(|b| !b.hidden)
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
            &catalogue(),
            &rule.target.browser_id,
            rule.target.profile_id.as_deref(),
            rule.private,
            &url,
        )
        .is_ok()
    {
        return;
    }

    let token;
    {
        let state = app.state::<Mutex<Pending>>();
        let Ok(mut held) = state.lock() else { return };
        held.url = Some(url.clone());
        held.source_app.clone_from(&source_app);
        held.token = held.token.wrapping_add(1);
        token = held.token;
    }

    let Some(window) = app.get_webview_window("picker") else {
        return;
    };
    let _ = window.emit("link:incoming", Incoming::new(url, source_app, token));
}

/// Pulled, not pushed: on a cold start `deliver` runs before the webview exists,
/// so an emitted link would land on nobody.
#[tauri::command]
fn picker_boot(state: tauri::State<'_, Mutex<Pending>>) -> Option<Incoming> {
    let held = state.lock().ok()?;
    Some(Incoming::new(
        held.url.clone()?,
        held.source_app.clone(),
        held.token,
    ))
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

/// The rule as the settings window shows it: what it covers, where it opens and
/// whether the browser it names is still installed.
#[derive(Serialize)]
struct RuleView {
    id: String,
    kind: &'static str,
    covers: String,
    browser: String,
    profile: Option<String>,
    icon: Option<String>,
    private: bool,
    source_app: Option<String>,
    resolved: bool,
}

fn describe(rule: &Rule, browsers: &[Browser]) -> RuleView {
    let found = browsers.iter().find(|b| b.id == rule.target.browser_id);
    let profile = rule.target.profile_id.as_ref().and_then(|id| {
        found
            .and_then(|b| b.profiles.iter().find(|p| &p.id == id))
            .map(|p| p.name.clone())
    });
    let (kind, covers) = match &rule.scope {
        Scope::Any => ("any", String::new()),
        Scope::Url(u) => ("url", u.clone()),
        Scope::Host(h) => ("host", h.clone()),
        Scope::Site(d) => ("site", d.clone()),
    };
    RuleView {
        id: rule.id.clone(),
        kind,
        covers,
        browser: found.map_or_else(|| rule.target.browser_id.clone(), |b| b.name.clone()),
        profile,
        icon: found.and_then(|b| icon_data(&b.exe, &b.id)),
        private: rule.private,
        source_app: rule.source_app.clone(),
        resolved: found.is_some(),
    }
}

#[tauri::command]
fn rules_list() -> Result<Vec<RuleView>, String> {
    let browsers = catalogue();
    Ok(store()
        .rules()
        .map_err(|e| e.to_string())?
        .rules
        .iter()
        .map(|r| describe(r, &browsers))
        .collect())
}

#[tauri::command]
fn rules_remove(id: String) -> Result<Vec<RuleView>, String> {
    let store = store();
    let mut rules = store.rules().map_err(|e| e.to_string())?;
    rules.remove(&id);
    store.save_rules(&rules).map_err(|e| e.to_string())?;
    rules_list()
}

#[tauri::command]
fn rules_reorder(ids: Vec<String>) -> Result<Vec<RuleView>, String> {
    let store = store();
    let mut rules = store.rules().map_err(|e| e.to_string())?;
    rules.reorder(&ids);
    store.save_rules(&rules).map_err(|e| e.to_string())?;
    rules_list()
}

#[derive(Serialize)]
struct BrowserView {
    id: String,
    name: String,
    exe: String,
    profiles: usize,
    private: bool,
    custom: bool,
    hidden: bool,
    icon: Option<String>,
}

fn seen(browser: &Browser) -> BrowserView {
    BrowserView {
        profiles: browser.profiles.len(),
        private: browser.private_flag.is_some(),
        icon: icon_data(&browser.exe, &browser.id),
        id: browser.id.clone(),
        name: browser.name.clone(),
        exe: browser.exe.clone(),
        custom: browser.custom,
        hidden: browser.hidden,
    }
}

#[tauri::command]
fn browsers_list() -> Vec<BrowserView> {
    catalogue().iter().map(seen).collect()
}

/// Saves the merged catalogue, which is what turns a detected browser into a
/// saved one the moment the user first touches it.
fn keep(browsers: Vec<Browser>) -> Result<Vec<BrowserView>, String> {
    let store = store();
    let mut config = store.browsers().unwrap_or_default();
    config.browsers = browsers;
    store.save_browsers(&config).map_err(|e| e.to_string())?;
    Ok(browsers_list())
}

#[tauri::command]
fn browsers_set_hidden(id: String, hidden: bool) -> Result<Vec<BrowserView>, String> {
    let mut all = catalogue();
    let Some(found) = all.iter_mut().find(|b| b.id == id) else {
        return Err("that browser is no longer installed".to_owned());
    };
    found.hidden = hidden;
    keep(all)
}

#[tauri::command]
fn browsers_remove(id: String) -> Result<Vec<BrowserView>, String> {
    let mut all = catalogue();
    if !all.iter().any(|b| b.id == id && b.custom) {
        return Err("only a browser you added can be removed".to_owned());
    }
    all.retain(|b| b.id != id);
    keep(all)
}

#[tauri::command]
fn browsers_add(name: String, exe: String, args: Vec<String>) -> Result<Vec<BrowserView>, String> {
    if !std::path::Path::new(&exe).is_file() {
        return Err("there is no program at that path".to_owned());
    }
    let mut all = catalogue();
    let id = format!("custom-{}", all.iter().filter(|b| b.custom).count() + 1);
    all.push(Browser {
        id,
        name,
        exe,
        profiles: Vec::new(),
        extra_args: args,
        private_flag: None,
        custom: true,
        hidden: false,
    });
    keep(all)
}

/// Everything the app decided on its own goes; what the user chose stays.
#[tauri::command]
fn maintenance_rescan() -> Result<Vec<BrowserView>, String> {
    let mut all = catalogue();
    all.retain(|b| b.custom);
    keep(all)
}

#[tauri::command]
fn maintenance_reset() -> Result<(), String> {
    let store = store();
    store
        .save_rules(&linkunbound_core::RuleSet::default())
        .map_err(|e| e.to_string())?;
    store
        .save_browsers(&Default::default())
        .map_err(|e| e.to_string())
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
    token: u64,
) -> Result<(), String> {
    let url = {
        let mut held = state.lock().map_err(|_| "state is poisoned".to_owned())?;
        if held.token != token {
            return Err("that link was replaced by a newer one".to_owned());
        }
        held.url
            .take()
            .ok_or_else(|| "no link pending".to_owned())?
    };

    launch::open(
        &catalogue(),
        &browser_id,
        profile_id.as_deref(),
        private,
        &url,
    )
    .map_err(|e| e.to_string())?;

    // Hidden before saving: a failed rule must not read as "pick somewhere else".
    shell::hide_picker(&app);

    if let Some(host) = host_of(&url)
        && let Some(scope) = remember.scope(&url, &host)
    {
        let store = store();
        let mut rules = store.rules().map_err(|e| e.to_string())?;
        rules.upsert(Rule {
            id: format!("{host}-{browser_id}-{}", rules.rules.len()),
            scope,
            source_app: None,
            target: Target {
                browser_id,
                profile_id,
            },
            private,
        });
        store.save_rules(&rules).map_err(|e| e.to_string())?;
    }
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
fn picker_dismiss(app: AppHandle, state: tauri::State<'_, Mutex<Pending>>) {
    if let Ok(mut held) = state.lock() {
        held.url = None;
    }
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
            rules_list,
            rules_remove,
            rules_reorder,
            browsers_list,
            browsers_set_hidden,
            browsers_remove,
            browsers_add,
            maintenance_rescan,
            maintenance_reset,
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

    /// `tauri-plugin-single-instance` reassembles argv on Windows by joining with
    /// `"|"` and splitting it back the same way, so a URL that legitimately
    /// contains a pipe in its path or query arrives as separate argv entries.
    #[test]
    fn a_url_containing_a_pipe_survives_the_single_instance_plugins_windows_split() {
        let whole = "https://intranet.corp/x?f=activo|urgente";
        let fragments: Vec<&str> = whole.split('|').collect();
        assert_eq!(fragments, ["https://intranet.corp/x?f=activo", "urgente"]);
        assert_eq!(link_from(&args(&fragments)).as_deref(), Some(whole));
    }

    #[test]
    fn remembering_the_whole_site_for_an_ip_never_captures_a_different_machine() {
        let scope = Remember::Site
            .scope("https://192.168.1.50/admin", "192.168.1.50")
            .unwrap();
        assert!(scope.matches("https://192.168.1.50/admin", "192.168.1.50"));
        assert!(!scope.matches("https://10.0.1.50/admin", "10.0.1.50"));
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
