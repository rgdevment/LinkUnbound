mod shell;
mod shop;
mod shortcut;
mod system;
mod update;

use std::sync::Mutex;

use linkunbound_core::{Browser, Language, Preferences, Rule, Scope, Store, Target, merge};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

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

/// The rule as the settings window shows it: what it covers, where it opens and
/// whether the browser it names is still installed.
#[derive(Serialize)]
struct RuleView {
    id: String,
    /// As stored, so the screen can offer to change it without matching names.
    browser_id: String,
    profile_id: Option<String>,
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
        browser_id: rule.target.browser_id.clone(),
        profile_id: rule.target.profile_id.clone(),
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

/// In place: remaking the rule would send it to the end of the order.
#[tauri::command]
fn rules_retarget(
    id: String,
    browser_id: String,
    profile_id: Option<String>,
) -> Result<Vec<RuleView>, String> {
    let target = Target {
        browser_id,
        profile_id,
    };
    let changed = store()
        .edit_rules(|rules| rules.retarget(&id, target.clone()))
        .map_err(|e| e.to_string())?;
    if !changed {
        return Err("that rule is no longer there".to_owned());
    }
    rules_list()
}

#[derive(Serialize)]
struct BrowserView {
    id: String,
    name: String,
    exe: String,
    profiles: usize,
    /// Named, not just counted: a rule can point at one profile of a browser,
    /// so the screen has to be able to offer them apart.
    profile_names: Vec<(String, String)>,
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
        profile_names: browser
            .profiles
            .iter()
            .map(|p| (p.id.clone(), p.name.clone()))
            .collect(),
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
    keep_in(&store(), browsers)?;
    Ok(browsers_list())
}

/// Refuses to write when the file cannot be read: going ahead erased every
/// custom browser the moment a switch was touched.
fn keep_in(store: &Store, browsers: Vec<Browser>) -> Result<(), String> {
    let mut config = store.browsers().map_err(|e| e.to_string())?;
    config.browsers = browsers;
    store.save_browsers(&config).map_err(|e| e.to_string())
}

#[tauri::command]
fn browsers_set_hidden(id: String, hidden: bool) -> Result<Vec<BrowserView>, String> {
    keep(hidden_in(catalogue(), &id, hidden)?)
}

fn hidden_in(mut all: Vec<Browser>, id: &str, hidden: bool) -> Result<Vec<Browser>, String> {
    let Some(found) = all.iter_mut().find(|b| b.id == id) else {
        return Err("that browser is no longer installed".to_owned());
    };
    found.hidden = hidden;
    Ok(all)
}

#[tauri::command]
fn browsers_remove(id: String) -> Result<Vec<BrowserView>, String> {
    keep(without(catalogue(), &id)?)
}

/// A detected browser is hidden, never removed: the next scan would bring it
/// back and the deletion would look like it failed.
fn without(mut all: Vec<Browser>, id: &str) -> Result<Vec<Browser>, String> {
    if !all.iter().any(|b| b.id == id && b.custom) {
        return Err("only a browser you added can be removed".to_owned());
    }
    all.retain(|b| b.id != id);
    Ok(all)
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

/// A detected browser can be renamed and given arguments or an icon, the way
/// 1.x allowed, but not moved: the path is what detection found, and a rescan
/// would overwrite anything typed over it.
#[tauri::command]
fn browsers_update(id: String, edit: Edit) -> Result<Vec<BrowserView>, String> {
    keep(edited(catalogue(), &id, edit, readable)?)
}

fn edited(
    mut all: Vec<Browser>,
    id: &str,
    edit: Edit,
    exists: impl Fn(&str) -> bool,
) -> Result<Vec<Browser>, String> {
    let Some(found) = all.iter_mut().find(|b| b.id == id) else {
        return Err("ese navegador ya no está".to_owned());
    };
    if found.custom {
        if !exists(&edit.exe) {
            return Err("no hay ningún programa en esa ruta".to_owned());
        }
        found.exe = edit.exe;
    }
    found.name = edit.name;
    found.extra_args = edit.args;
    found.private_flag = edit.private_flag.filter(|f| !f.is_empty());
    found.icon_path = edit.icon_path.filter(|p| !p.is_empty());
    Ok(all)
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
    keep(ordered(catalogue(), &ids))
}

/// Anything the caller did not name keeps its place at the end: a browser
/// detected since the screen was drawn must not fall off the list.
fn ordered(mut all: Vec<Browser>, ids: &[String]) -> Vec<Browser> {
    let mut moved: Vec<Browser> = Vec::with_capacity(all.len());
    for id in ids {
        if let Some(at) = all.iter().position(|b| &b.id == id) {
            moved.push(all.remove(at));
        }
    }
    moved.append(&mut all);
    moved
}

/// Everything the app decided on its own goes; what the user chose stays.
#[tauri::command]
fn maintenance_rescan() -> Result<Vec<BrowserView>, String> {
    keep(only_mine(catalogue()))
}

/// Forgets what detection found so the next read picks it up again. What the
/// user added by hand is not detectable, so dropping it would be a deletion.
fn only_mine(mut all: Vec<Browser>) -> Vec<Browser> {
    all.retain(|b| b.custom);
    all
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

const HERE: &str = env!("CARGO_PKG_VERSION");

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Build {
    version: &'static str,
    license: &'static str,
    repository: &'static str,
    candidates: bool,
    candidates_apply: bool,
}

#[tauri::command]
fn about() -> Build {
    let kept = update::looked(store().dir());
    Build {
        version: HERE,
        license: "GPL-3.0-only",
        repository: "https://github.com/rgdevment/LinkUnbound",
        candidates: update::tracking(HERE, kept.candidates),
        candidates_apply: update::route().route != update::Route::Store,
    }
}

#[cfg(windows)]
fn owner(app: &AppHandle) -> Option<isize> {
    app.get_webview_window("settings")
        .or_else(|| app.webview_windows().into_values().next())
        .and_then(|window| window.hwnd().ok())
        .map(|window| window.0 as isize)
}

#[cfg(not(windows))]
const fn owner(_app: &AppHandle) -> Option<isize> {
    None
}

/// A copy the store keeps never reads the manifest. It cannot install from a download, so a
/// version the manifest names is one it could be told about and never take — and the offer would
/// arrive stripped of `installs`, leaving a notice with no button under it.
async fn from_the_store(
    app: &AppHandle,
    dir: &std::path::Path,
    looked: &mut update::Looked,
    now: u64,
    asked: bool,
) -> Option<update::Ready> {
    let remembered = |looked: &update::Looked| {
        looked
            .found_version
            .as_deref()
            .and_then(|version| update::from_the_shop(version, HERE))
    };

    // Held to the manifest's interval too: a Store that never answers strands the thread.
    if !asked && !update::due(looked.checked_at, now) {
        return remembered(looked);
    }
    let window = owner(app)?;
    let Ok(shelf) = tauri::async_runtime::spawn_blocking(move || shop::asked(window)).await else {
        return remembered(looked);
    };

    match shelf {
        // Not knowing is not being current, so the old wording stands and the interval is left
        // alone: writing it would buy the silence another day.
        shop::Shelf::Silent => remembered(looked),
        shop::Shelf::Current => {
            looked.checked_at = Some(now);
            looked.found_version = None;
            update::keep(dir, looked);
            None
        }
        shop::Shelf::Waiting(version) => {
            let seen = update::from_the_shop(&version, HERE);
            looked.checked_at = Some(now);
            looked.found_version = seen.as_ref().map(|one| one.version.clone());
            update::keep(dir, looked);
            seen
        }
    }
}

#[tauri::command]
async fn update_ready(
    app: AppHandle,
    now_please: Option<bool>,
) -> Result<Option<update::Ready>, String> {
    let dir = store().dir().to_path_buf();
    let kept = update::route();
    let mut looked = update::looked(&dir);
    let now = update::now();
    let asked = now_please.unwrap_or(false);

    if kept.route == update::Route::Store {
        return Ok(from_the_store(&app, &dir, &mut looked, now, asked).await);
    }

    if !asked && !update::due(looked.checked_at, now) {
        return Ok(update::remembered(
            HERE,
            looked.found_version.as_deref(),
            kept,
        ));
    }

    let manifest = tauri::async_runtime::spawn_blocking(update::fetch)
        .await
        .map_err(|why| why.to_string())?;

    // A look that never answered says nothing about whether an update is owed, so what was found
    // before stays where it is.
    let Some(manifest) = manifest else {
        return Ok(update::remembered(
            HERE,
            looked.found_version.as_deref(),
            kept,
        ));
    };

    let seen = update::newer(HERE, &manifest, kept, looked.candidates);
    looked.checked_at = Some(now);
    looked.found_version = seen.as_ref().map(|one| one.version.clone());
    update::keep(&dir, &looked);
    Ok(seen)
}

/// The store never carries candidates, so a copy kept there is not offered the choice.
#[tauri::command]
fn update_candidates(wants: bool) -> Result<(), String> {
    let dir = store().dir().to_path_buf();
    let mut looked = update::looked(&dir);
    looked.candidates = Some(wants);
    // What the old track found is not an offer on the new one.
    looked.checked_at = None;
    looked.found_version = None;
    update::keep(&dir, &looked);
    Ok(())
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Underway {
    stage: &'static str,
    far: u64,
}

#[tauri::command]
async fn update_install(app: AppHandle, busy: tauri::State<'_, Updating>) -> Result<(), String> {
    use tauri_plugin_updater::UpdaterExt;

    let _held = busy
        .inner()
        .0
        .claim()
        .ok_or_else(|| "updateBusy".to_owned())?;

    let kept = update::route();
    if kept.route == update::Route::Store {
        return take_from_the_shop(app).await;
    }
    if !update::self_installs(kept.route) || update::from_a_mount() {
        return Err("updateNotHere".to_owned());
    }

    let dir = store().dir().to_path_buf();
    let found = update::looked(&dir).found_version;
    let Some(want) = update::remembered(HERE, found.as_deref(), kept).map(|one| one.version) else {
        return Err("updateGone".to_owned());
    };

    let asked = want.clone();
    let update = app
        .updater_builder()
        .endpoints(vec![
            update::channel_for(&want)
                .parse()
                .map_err(|_| "updateFailed".to_owned())?,
        ])
        .map_err(|why| why.to_string())?
        // Pinned to what the person was shown, so a feed that moves in between cannot quietly
        // hand them a different version than the one they agreed to.
        .version_comparator(move |_, release| release.version.to_string() == asked)
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|why| why.to_string())?
        .check()
        .await
        .map_err(|why| why.to_string())?;

    let Some(mut update) = update else {
        return Err("updateGone".to_owned());
    };

    // The feed names the address the installer comes from, so it is checked against where our
    // releases actually live before a single byte is asked for.
    if !update::ours(update.download_url.as_str(), &want) {
        return Err("updateElsewhere".to_owned());
    }
    // The plugin builds the download with no deadline of its own, and a server that dribbles
    // bytes forever would otherwise be waited on forever.
    update.timeout = Some(std::time::Duration::from_secs(600));

    let telling = app.clone();
    let done = app.clone();
    let mut carried: u64 = 0;
    let mut said = 0;
    update
        .download_and_install(
            move |chunk, whole| {
                // The callback hands over the length of one chunk, not how much has arrived.
                carried += chunk as u64;
                let far = whole.map_or(0, |all| carried * 100 / all.max(1));
                if far != said {
                    said = far;
                    let _ = telling.emit(
                        "updating",
                        Underway {
                            stage: "getting",
                            far,
                        },
                    );
                }
            },
            // The last thing anyone sees on Windows: the installer takes the process with it and
            // nothing after the await ever runs.
            move || {
                let _ = done.emit(
                    "updating",
                    Underway {
                        stage: "installing",
                        far: 100,
                    },
                );
            },
        )
        .await
        .map_err(|why| why.to_string())?;

    let handle = app.clone();
    app.run_on_main_thread(move || handle.restart())
        .map_err(|why| why.to_string())
}

/// Windows ends the process to put the new package in place, so the progress left behind is the
/// last thing anyone sees.
async fn take_from_the_shop(app: AppHandle) -> Result<(), String> {
    let window = owner(&app).ok_or_else(|| "updateNotHere".to_owned())?;
    let telling = app.clone();
    let taken = tauri::async_runtime::spawn_blocking(move || {
        let mut said: (&'static str, u64) = ("", 0);
        shop::take(window, move |stage, far| {
            if said != (stage, far) {
                said = (stage, far);
                let _ = telling.emit("updating", Underway { stage, far });
            }
        })
    })
    .await
    .map_err(|why| why.to_string())?;

    match taken {
        Ok(()) => Ok(()),
        Err(shop::Trouble::Gone) => Err("updateGone".to_owned()),
        Err(shop::Trouble::Stopped) => Err("updateStopped".to_owned()),
        Err(shop::Trouble::Failed(why)) => Err(why),
    }
}

/// A second press while the first is still downloading would run two installs over each other.
#[derive(Default)]
struct Updating(OneAtATime);

#[derive(Default)]
struct OneAtATime(std::sync::atomic::AtomicBool);

impl OneAtATime {
    fn claim(&self) -> Option<Releasing<'_>> {
        use std::sync::atomic::Ordering;
        self.0
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
            .then(|| Releasing(&self.0))
    }
}

struct Releasing<'a>(&'a std::sync::atomic::AtomicBool);

impl Drop for Releasing<'_> {
    fn drop(&mut self) {
        self.0.store(false, std::sync::atomic::Ordering::Release);
    }
}

pub fn run() {
    tauri::Builder::default()
        .manage(Mutex::new(Held::default()))
        .manage(Updating::default())
        .invoke_handler(tauri::generate_handler![
            rules_list,
            rules_remove,
            rules_retarget,
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
            system_repair,
            about,
            update_ready,
            update_install,
            update_candidates
        ])
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
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

            // The uninstaller hands the registration back before taking the binary away, so this
            // runs instead of the reconcile, never after it.
            if args.iter().any(|a| a == "--unregister") {
                let _ = system::set_registered(false);
                app.handle().exit(0);
                return Ok(());
            }

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
    use super::{
        Edit, Store, describe, edited, free_id, hidden_in, keep_in, only_mine, ordered, seen,
        spoken, without,
    };
    use linkunbound_core::{Locale, Preferences, Profile, Rule, Scope, Target};

    fn detected(id: &str) -> linkunbound_core::Browser {
        linkunbound_core::Browser {
            id: id.to_owned(),
            name: "Google Chrome".to_owned(),
            exe: "chrome.exe".to_owned(),
            profiles: vec![Profile {
                id: "Default".to_owned(),
                name: "Personal".to_owned(),
                args: Vec::new(),
            }],
            extra_args: Vec::new(),
            private_flag: Some("--incognito".to_owned()),
            icon_path: None,
            custom: false,
            hidden: false,
        }
    }

    fn rule_for(browser: &str, profile: Option<&str>) -> Rule {
        Rule {
            id: "r1".to_owned(),
            scope: Scope::Site("github.com".to_owned()),
            source_app: None,
            target: Target {
                browser_id: browser.to_owned(),
                profile_id: profile.map(str::to_owned),
            },
            private: false,
        }
    }

    #[test]
    fn a_rule_pointing_at_a_browser_that_is_gone_is_not_called_resolved() {
        let installed = [detected("chrome")];

        let live = describe(&rule_for("chrome", Some("Default")), &installed);
        assert!(live.resolved);
        assert_eq!(live.browser, "Google Chrome");
        assert_eq!(live.profile.as_deref(), Some("Personal"));

        let gone = describe(&rule_for("brave", None), &installed);
        assert!(
            !gone.resolved,
            "an uninstalled browser must not read as live"
        );
        assert_eq!(gone.browser, "brave", "and it still names what it was");
    }

    #[test]
    fn a_rule_carries_the_destination_it_was_saved_with() {
        let view = describe(&rule_for("chrome", Some("Default")), &[detected("chrome")]);
        assert_eq!(view.browser_id, "chrome");
        assert_eq!(view.profile_id.as_deref(), Some("Default"));
    }

    #[test]
    fn a_detected_browser_is_not_reported_as_one_the_user_added() {
        let view = seen(&detected("chrome"));
        assert!(!view.custom);
        assert_eq!(view.profiles, 1);
        assert_eq!(
            view.profile_names,
            vec![("Default".to_owned(), "Personal".to_owned())],
            "named, so a rule can be pointed at one profile"
        );
        assert!(view.private, "it declares a private switch");

        let mine = seen(&custom("custom-1"));
        assert!(mine.custom);
        assert!(!mine.private);
    }

    /// `Locale::System` must mean the same in both windows, so it is decided
    /// here rather than in each webview.
    #[test]
    fn the_language_handed_to_the_screen_obeys_the_explicit_choice() {
        let spanish = Preferences {
            locale: Locale::Spanish,
            ..Preferences::default()
        };
        let english = Preferences {
            locale: Locale::English,
            ..Preferences::default()
        };
        assert_eq!(spoken(&spanish), "es");
        assert_eq!(spoken(&english), "en");
    }

    fn scratch(name: &str) -> std::path::PathBuf {
        let dir =
            std::env::temp_dir().join(format!("linkunbound-cmd-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("should create");
        dir
    }

    /// `SCHEMA_VERSION` exists so a newer file is recognised rather than parsed.
    /// Writing over it anyway erases the fields this version cannot read.
    #[test]
    fn a_file_this_version_cannot_read_is_not_written_over() {
        let dir = scratch("unreadable");
        let store = Store::at(&dir);
        std::fs::write(
            dir.join("browsers.json"),
            r#"{"schema_version": 99, "browsers": []}"#,
        )
        .expect("should write");

        let before = std::fs::read(dir.join("browsers.json")).expect("should read");
        assert!(keep_in(&store, vec![custom("custom-1")]).is_err());
        assert_eq!(
            std::fs::read(dir.join("browsers.json")).expect("should read"),
            before,
            "the file must be left exactly as it was"
        );
    }

    #[test]
    fn what_the_user_added_survives_a_rescan() {
        let all = vec![detected("chrome"), custom("custom-1"), detected("firefox")];
        let kept = only_mine(all);
        assert_eq!(kept.len(), 1);
        assert_eq!(kept[0].id, "custom-1", "detection finds the rest again");
    }

    fn edit_of(name: &str, exe: &str) -> Edit {
        Edit {
            name: name.to_owned(),
            exe: exe.to_owned(),
            args: vec!["--new-window".to_owned()],
            private_flag: Some(String::new()),
            icon_path: None,
        }
    }

    /// Detection owns the path of what it found: a rescan would put it back, so
    /// letting it be typed over promises an edit that does not survive.
    #[test]
    fn only_a_browser_the_user_added_may_move_its_path() {
        let all = vec![detected("chrome"), custom("custom-1")];

        let touched = edited(
            all.clone(),
            "chrome",
            edit_of("Trabajo", "moved.exe"),
            |_| true,
        )
        .expect("a detected browser may still be renamed");
        assert_eq!(touched[0].name, "Trabajo");
        assert_eq!(touched[0].exe, "chrome.exe", "but not moved");
        assert_eq!(
            touched[0].private_flag, None,
            "an empty switch clears it rather than saving a blank"
        );

        let mine = edited(all, "custom-1", edit_of("Mío", "other.exe"), |_| true)
            .expect("one the user added may move");
        assert_eq!(mine[1].exe, "other.exe");
    }

    #[test]
    fn an_edit_pointing_nowhere_is_refused_before_it_is_saved() {
        let all = vec![custom("custom-1")];
        assert!(
            edited(
                all.clone(),
                "custom-1",
                edit_of("Mío", "ghost.exe"),
                |_| false
            )
            .is_err()
        );
        assert!(edited(all, "gone", edit_of("Mío", "x.exe"), |_| true).is_err());
    }

    /// Removing a detected browser would last until the next scan, so the screen
    /// offers hiding instead and the command refuses.
    #[test]
    fn a_detected_browser_cannot_be_removed_only_hidden() {
        let all = vec![detected("chrome"), custom("custom-1")];

        assert!(without(all.clone(), "chrome").is_err());

        let hidden = hidden_in(all.clone(), "chrome", true).expect("hiding is allowed");
        assert!(hidden[0].hidden);
        let shown = hidden_in(hidden, "chrome", false).expect("and so is showing it again");
        assert!(!shown[0].hidden, "the switch has to work both ways");

        let gone = without(all, "custom-1").expect("one the user added may go");
        assert_eq!(gone.len(), 1);
        assert_eq!(gone[0].id, "chrome");
    }

    #[test]
    fn hiding_something_that_is_not_there_says_so() {
        assert!(hidden_in(vec![detected("chrome")], "brave", true).is_err());
    }

    /// The order decides which browser sits under the first key of the picker.
    /// Anything the screen did not know about keeps its place at the end.
    #[test]
    fn reordering_keeps_what_the_caller_never_named() {
        let all = vec![detected("chrome"), custom("custom-1"), detected("firefox")];
        let asked = vec!["custom-1".to_owned(), "chrome".to_owned()];

        let moved = ordered(all, &asked);
        let ids: Vec<&str> = moved.iter().map(|b| b.id.as_str()).collect();
        assert_eq!(
            ids,
            ["custom-1", "chrome", "firefox"],
            "the unnamed one survives at the end rather than falling off"
        );
    }

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
