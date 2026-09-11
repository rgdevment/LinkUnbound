mod shell;
mod shortcut;
mod system;

use std::sync::Mutex;

use linkunbound_core::{Browser, Language, Preferences, Rule, Scope, Store, merge};
use serde::Serialize;
use tauri::{AppHandle, Manager};

/// The combination actually claimed, which the settings screen needs and only
/// the registration knows.
#[derive(Default)]
struct Held {
    shortcut: Option<String>,
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
    let path = linkunbound_win::icon_for(exe, id, &icons_dir(), linkunbound_shell::ICON_SIDE)?;
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

/// Where the 1.x line kept its files, so an upgrade finds them in place.
fn store() -> Store {
    let base = std::env::var_os("LOCALAPPDATA")
        .or_else(|| std::env::var_os("APPDATA"))
        .map_or_else(std::env::temp_dir, std::path::PathBuf::from);
    Store::at(base.join("LinkUnbound"))
}

/// Detection plus what the user saved, which is the only place the two meet.
fn catalogue() -> Vec<Browser> {
    let saved = store().browsers().map(|c| c.browsers).unwrap_or_default();
    merge(system::browsers(), &saved)
}

/// Refuses to go on when the saved file cannot be read. `SCHEMA_VERSION` exists
/// so a newer file is recognised rather than parsed; swallowing that error and
/// writing anyway erased every custom browser the moment a switch was touched.
fn saved_browsers() -> Result<linkunbound_core::BrowserConfig, String> {
    store().browsers().map_err(|e| e.to_string())
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
    store()
        .edit_rules(|rules| rules.remove(&id))
        .map_err(|e| e.to_string())?;
    rules_list()
}

#[tauri::command]
fn rules_reorder(ids: Vec<String>) -> Result<Vec<RuleView>, String> {
    store()
        .edit_rules(|rules| {
            rules.reorder(&ids);
            true
        })
        .map_err(|e| e.to_string())?;
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
    args: Vec<String>,
    private_flag: Option<String>,
    icon_path: Option<String>,
}

fn seen(browser: &Browser) -> BrowserView {
    BrowserView {
        profiles: browser.profiles.len(),
        private: browser.private_flag.is_some(),
        icon: icon_data(browser.icon_source(), &browser.id),
        id: browser.id.clone(),
        name: browser.name.clone(),
        exe: browser.exe.clone(),
        custom: browser.custom,
        hidden: browser.hidden,
        args: browser.extra_args.clone(),
        private_flag: browser.private_flag.clone(),
        icon_path: browser.icon_path.clone(),
    }
}

/// What the settings screen can change about a browser it owns.
#[derive(serde::Deserialize)]
struct Edit {
    name: String,
    exe: String,
    args: Vec<String>,
    private_flag: Option<String>,
    icon_path: Option<String>,
}

fn readable(path: &str) -> bool {
    std::path::Path::new(path).is_file()
}

#[tauri::command]
fn browsers_list() -> Vec<BrowserView> {
    catalogue().iter().map(seen).collect()
}

/// Saves the merged catalogue, which is what turns a detected browser into a
/// saved one the moment the user first touches it.
fn keep(browsers: Vec<Browser>) -> Result<Vec<BrowserView>, String> {
    let mut config = saved_browsers()?;
    config.browsers = browsers;
    store().save_browsers(&config).map_err(|e| e.to_string())?;
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

/// Never reuses a number a live entry already holds, or editing one would
/// rename the other.
fn free_id(all: &[Browser]) -> String {
    (1..)
        .map(|n| format!("custom-{n}"))
        .find(|id| !all.iter().any(|b| &b.id == id))
        .unwrap_or_else(|| "custom-1".to_owned())
}

#[tauri::command]
fn browsers_add(edit: Edit) -> Result<Vec<BrowserView>, String> {
    if !readable(&edit.exe) {
        return Err("no hay ningún programa en esa ruta".to_owned());
    }
    let mut all = catalogue();
    all.push(Browser {
        id: free_id(&all),
        name: edit.name,
        exe: edit.exe,
        profiles: Vec::new(),
        extra_args: edit.args,
        private_flag: edit.private_flag.filter(|f| !f.is_empty()),
        icon_path: edit.icon_path.filter(|p| !p.is_empty()),
        custom: true,
        hidden: false,
    });
    keep(all)
}

#[tauri::command]
fn browsers_update(id: String, edit: Edit) -> Result<Vec<BrowserView>, String> {
    if !readable(&edit.exe) {
        return Err("no hay ningún programa en esa ruta".to_owned());
    }
    let mut all = catalogue();
    let Some(found) = all.iter_mut().find(|b| b.id == id && b.custom) else {
        return Err("solo puedes editar un navegador que añadiste tú".to_owned());
    };
    found.name = edit.name;
    found.exe = edit.exe;
    found.extra_args = edit.args;
    found.private_flag = edit.private_flag.filter(|f| !f.is_empty());
    found.icon_path = edit.icon_path.filter(|p| !p.is_empty());
    keep(all)
}

#[tauri::command]
fn browsers_duplicate(id: String) -> Result<Vec<BrowserView>, String> {
    let mut all = catalogue();
    let Some(source) = all.iter().find(|b| b.id == id) else {
        return Err("ese navegador ya no está".to_owned());
    };
    let copy = source.duplicated(free_id(&all));
    let at = all
        .iter()
        .position(|b| b.id == id)
        .map_or(all.len(), |i| i + 1);
    all.insert(at, copy);
    keep(all)
}

/// The order here is the order of the picker, so moving one is how the user
/// decides which destination sits under the first key.
#[tauri::command]
fn browsers_reorder(ids: Vec<String>) -> Result<Vec<BrowserView>, String> {
    let mut all = catalogue();
    let mut moved: Vec<Browser> = Vec::with_capacity(all.len());
    for id in &ids {
        if let Some(at) = all.iter().position(|b| &b.id == id) {
            moved.push(all.remove(at));
        }
    }
    moved.append(&mut all);
    keep(moved)
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

/// The shortcut the app really holds, which is not always the one asked for.
#[derive(Serialize)]
struct Settings {
    prefs: Preferences,
    shortcut_held: Option<String>,
    /// Resolved here so `Locale::System` cannot mean two things at once.
    language: &'static str,
}

fn claim(app: &AppHandle, prefs: &Preferences) -> Settings {
    let shortcut_held = shortcut::install(app, prefs.shortcut.as_deref());
    if let Ok(mut held) = app.state::<Mutex<Held>>().lock() {
        held.shortcut.clone_from(&shortcut_held);
    }
    Settings {
        language: spoken(prefs),
        prefs: prefs.clone(),
        shortcut_held,
    }
}

/// Reads without touching the registration: the picker asks for this too, and
/// re-claiming the combination on every link would be gratuitous.
#[tauri::command]
fn prefs_get(state: tauri::State<'_, Mutex<Held>>) -> Settings {
    let prefs = store().prefs();
    let shortcut_held = state.lock().ok().and_then(|h| h.shortcut.clone());
    Settings {
        language: spoken(&prefs),
        prefs,
        shortcut_held,
    }
}

fn spoken(prefs: &Preferences) -> &'static str {
    match Language::chosen(prefs.locale) {
        Language::Spanish => "es",
        Language::English => "en",
    }
}

#[tauri::command]
fn prefs_set(app: AppHandle, prefs: Preferences) -> Result<Settings, String> {
    if !prefs.reachable() {
        return Err("unreachable".to_owned());
    }
    store().save_prefs(&prefs).map_err(|e| e.to_string())?;
    Ok(claim(&app, &prefs))
}

#[tauri::command]
fn system_open_default_apps() -> Result<(), String> {
    system::open_default_apps()
}

/// Re-registers against this executable, which is the only repair there is: the
/// command pointed somewhere this binary no longer lives.
#[tauri::command]
fn system_repair() -> Result<system::SystemState, String> {
    system::set_registered(true)
}

#[tauri::command]
fn maintenance_report() -> Result<String, String> {
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
    let body = linkunbound_core::diagnostics(
        env!("CARGO_PKG_VERSION"),
        &facts,
        &store.rules().unwrap_or_default(),
        &store.prefs(),
    );

    let target = std::env::var_os("USERPROFILE")
        .map_or_else(std::env::temp_dir, std::path::PathBuf::from)
        .join("Desktop")
        .join("linkunbound-diagnostico.md");
    let target = if target.parent().is_some_and(std::path::Path::is_dir) {
        target
    } else {
        std::env::temp_dir().join("linkunbound-diagnostico.md")
    };
    std::fs::write(&target, body).map_err(|e| e.to_string())?;
    Ok(target.to_string_lossy().into_owned())
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

pub fn run() {
    tauri::Builder::default()
        .manage(Mutex::new(Held::default()))
        .invoke_handler(tauri::generate_handler![
            rules_list,
            rules_remove,
            rules_reorder,
            browsers_list,
            browsers_set_hidden,
            browsers_remove,
            browsers_add,
            browsers_update,
            browsers_duplicate,
            browsers_reorder,
            maintenance_rescan,
            maintenance_reset,
            maintenance_report,
            prefs_get,
            prefs_set,
            system_state,
            system_set_registered,
            system_set_startup,
            system_open_default_apps,
            system_repair
        ])
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        // Two tray clicks used to mean two processes, each writing the registry
        // and each claiming the shortcut. The second now raises the first.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            shell::open_settings(app);
        }))
        .setup(|app| {
            let args: Vec<String> = std::env::args_os()
                .map(|a| a.to_string_lossy().into_owned())
                .collect();
            system::reconcile();

            if args.iter().any(|a| a == "--register") {
                app.handle().exit(0);
                return Ok(());
            }
            // The resident owns the tray and the shortcut; this binary is only
            // the settings window, opened and closed on demand.
            claim(app.handle(), &store().prefs());
            shell::open_settings(app.handle());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("tauri failed to start");
}

#[cfg(test)]
mod tests {
    use super::free_id;

    fn custom(id: &str) -> linkunbound_core::Browser {
        linkunbound_core::Browser {
            id: id.to_owned(),
            name: id.to_owned(),
            exe: "x.exe".to_owned(),
            profiles: Vec::new(),
            extra_args: Vec::new(),
            private_flag: None,
            icon_path: None,
            custom: true,
            hidden: false,
        }
    }

    /// Reusing a number a live entry holds would make an edit rename the other.
    #[test]
    fn a_new_entry_never_takes_an_id_another_one_already_holds() {
        let mut all = vec![custom("custom-1"), custom("custom-3")];
        assert_eq!(free_id(&all), "custom-2");
        all.push(custom("custom-2"));
        assert_eq!(free_id(&all), "custom-4");
    }

    #[test]
    fn the_first_added_browser_starts_the_numbering() {
        assert_eq!(free_id(&[]), "custom-1");
    }
}
