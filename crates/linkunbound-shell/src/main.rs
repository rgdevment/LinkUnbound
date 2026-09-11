#![windows_subsystem = "windows"]

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::mpsc::channel;
use std::time::Duration;

use linkunbound_core::{Language, Rule, Store, Strings, Target, host_of, normalise};
use linkunbound_shell::tray::{Asked, Tray};
use linkunbound_shell::{ICON_SIDE, Listed, Notice, Picker, Reaches, dress, single};
use slint::ComponentHandle;

fn store() -> Store {
    let base = std::env::var_os("LOCALAPPDATA")
        .or_else(|| std::env::var_os("APPDATA"))
        .map_or_else(std::env::temp_dir, std::path::PathBuf::from);
    Store::at(base.join("LinkUnbound"))
}

/// Every separator inside the single argument the shell passes is part of the link.
fn link_from(args: &[String]) -> Option<String> {
    if args.len() < 2 {
        return None;
    }
    normalise(&args[1..].join(" ")).or_else(|| args.iter().skip(1).find_map(|a| normalise(a)))
}

#[cfg(windows)]
mod host {
    use linkunbound_core::Browser;

    pub fn browsers() -> Vec<Browser> {
        linkunbound_win::installed_browsers()
    }

    pub fn icon(browser: &Browser) -> Option<String> {
        let dir = std::env::var_os("LOCALAPPDATA")
            .map_or_else(std::env::temp_dir, std::path::PathBuf::from)
            .join("LinkUnbound")
            .join("icons");
        linkunbound_win::icon_for(browser.icon_source(), &browser.id, &dir, super::ICON_SIDE)
            .map(|p| p.to_string_lossy().into_owned())
    }

    pub fn clicked_in() -> Option<String> {
        linkunbound_win::source_app()
    }

    pub fn light_taskbar() -> bool {
        linkunbound_win::taskbar_is_light()
    }
}

#[cfg(not(windows))]
mod host {
    use linkunbound_core::Browser;

    pub fn browsers() -> Vec<Browser> {
        Vec::new()
    }

    pub fn icon(_browser: &Browser) -> Option<String> {
        None
    }

    pub fn clicked_in() -> Option<String> {
        None
    }

    pub fn light_taskbar() -> bool {
        false
    }
}

fn catalogue() -> Vec<linkunbound_core::Browser> {
    let saved = store().browsers().map(|c| c.browsers).unwrap_or_default();
    linkunbound_core::merge(host::browsers(), &saved)
}

fn rows() -> Vec<Listed> {
    let browsers = catalogue();
    let mut listed = linkunbound_shell::destinations(&browsers);
    for row in &mut listed {
        if let Some(found) = browsers.iter().find(|b| b.id == row.browser_id) {
            row.icon = host::icon(found);
        }
    }
    listed
}

struct Fired {
    rule_id: String,
    browser: String,
    host: String,
}

/// A rule that cannot be honoured falls through to the picker, never elsewhere.
fn answered_by_rule(url: &str, source: Option<&str>) -> Option<Fired> {
    let rules = store().rules().ok()?;
    let host = host_of(url)?;
    let browsers = catalogue();
    let rule = rules.resolve(url, &host, source)?;
    linkunbound_core::launch(
        &browsers,
        &rule.target.browser_id,
        rule.target.profile_id.as_deref(),
        rule.private,
        url,
    )
    .ok()?;
    Some(Fired {
        rule_id: rule.id.clone(),
        browser: browsers
            .iter()
            .find(|b| b.id == rule.target.browser_id)
            .map_or_else(|| rule.target.browser_id.clone(), |b| b.name.clone()),
        host,
    })
}

const NOTICE_SECONDS: i32 = 6;

/// Never focused: it must not take the keyboard from whatever is being done.
fn flash(notice: &Notice, words: &Strings, fired: &Fired) {
    notice.set_headline(Strings::fill(words.notice_opened, &fired.browser).into());
    notice.set_reason(Strings::fill(words.notice_by_rule, &fired.host).into());
    notice.set_undo_label(words.notice_undo.into());
    notice.set_left(NOTICE_SECONDS);
    let _ = notice.show();
}

/// Settings runs in another process: the file is the only channel between them.
fn prefs_touched_at() -> Option<std::time::SystemTime> {
    let base = std::env::var_os("LOCALAPPDATA")
        .or_else(|| std::env::var_os("APPDATA"))
        .map_or_else(std::env::temp_dir, std::path::PathBuf::from);
    std::fs::metadata(base.join("LinkUnbound").join("preferences.json"))
        .and_then(|m| m.modified())
        .ok()
}

fn forget(rule_id: &str) {
    let store = store();
    let Ok(mut rules) = store.rules() else { return };
    rules.remove(rule_id);
    let _ = store.save_rules(&rules);
}

fn remember(url: &str, chosen: &Listed, private: bool, reach: Reaches) {
    let Some(host) = host_of(url) else { return };
    let Some(scope) = reach.scope(url, &host) else {
        return;
    };
    let store = store();
    let Ok(mut rules) = store.rules() else { return };
    rules.upsert(Rule {
        id: String::new(),
        scope,
        source_app: None,
        target: Target {
            browser_id: chosen.browser_id.clone(),
            profile_id: chosen.profile_id.clone(),
        },
        private,
    });
    let _ = store.save_rules(&rules);
}

fn open_settings() {
    let Ok(here) = std::env::current_exe() else {
        return;
    };
    let beside = here.with_file_name(if cfg!(windows) {
        "linkunbound-settings.exe"
    } else {
        "linkunbound-settings"
    });
    let _ = std::process::Command::new(beside).spawn();
}

#[derive(Default)]
struct Shown {
    url: Option<String>,
    rows: Vec<Listed>,
}

fn present(picker: &Picker, words: &Strings, shown: &Rc<RefCell<Shown>>, url: String) {
    let listed = rows();
    if listed.is_empty() {
        open_settings();
        return;
    }
    dress(picker, words, &url, host::clicked_in().as_deref(), &listed);
    {
        let mut held = shown.borrow_mut();
        held.url = Some(url);
        held.rows = listed;
    }
    let _ = picker.show();
}

fn main() -> Result<(), slint::PlatformError> {
    let args: Vec<String> = std::env::args_os()
        .map(|a| a.to_string_lossy().into_owned())
        .collect();
    let incoming = link_from(&args);

    let (links, inbox) = channel::<String>();
    let Some(_server) = single::claim(links) else {
        if let Some(url) = incoming {
            single::hand_over(&url);
        } else {
            open_settings();
        }
        return Ok(());
    };

    let picker = Picker::new()?;
    let shown = Rc::new(RefCell::new(Shown::default()));
    let words = Rc::new(Cell::new(
        Language::chosen(store().prefs().locale).strings(),
    ));

    {
        let handle = picker.as_weak();
        picker.on_dismissed(move || {
            if let Some(window) = handle.upgrade() {
                let _ = window.hide();
            }
        });
    }
    {
        let handle = picker.as_weak();
        let shown = Rc::clone(&shown);
        let spoken = Rc::clone(&words);
        picker.on_open(move |index, private| {
            let Some(window) = handle.upgrade() else {
                return;
            };
            let reach = Reaches::at(window.get_reach_index());
            let held = shown.borrow();
            let Some(url) = held.url.clone() else { return };
            let Some(chosen) = held.rows.get(usize::try_from(index).unwrap_or(0)) else {
                return;
            };
            match linkunbound_core::launch(
                &catalogue(),
                &chosen.browser_id,
                chosen.profile_id.as_deref(),
                private,
                &url,
            ) {
                Ok(()) => {
                    let _ = window.hide();
                    remember(&url, chosen, private, reach);
                }
                Err(why) => window.set_problem(spoken.get().on_failure(&why).into()),
            }
        });
    }
    {
        let handle = picker.as_weak();
        let shown = Rc::clone(&shown);
        picker.on_copy(move || {
            if let Some(window) = handle.upgrade()
                && shown.borrow().url.is_some()
            {
                window.set_copied(true);
            }
        });
    }

    if let Some(url) = incoming {
        present(&picker, &words.get(), &shown, url);
    }

    let notice = Notice::new()?;
    let firing: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));
    {
        let handle = notice.as_weak();
        let firing = Rc::clone(&firing);
        notice.on_undo(move || {
            if let Some(id) = firing.borrow_mut().take() {
                forget(&id);
            }
            if let Some(window) = handle.upgrade() {
                let _ = window.hide();
            }
        });
    }
    {
        let handle = notice.as_weak();
        notice.on_dismissed(move || {
            if let Some(window) = handle.upgrade() {
                let _ = window.hide();
            }
        });
    }

    let tray = Tray::install(host::light_taskbar(), &words.get());
    let (asks, tray_inbox) = channel::<Asked>();

    // Slint owns the UI thread, so the channels drain from its timer.
    let ticker = slint::Timer::default();
    let handle = picker.as_weak();
    let notice_handle = notice.as_weak();
    let mut ticks = 0u32;
    let mut prefs_seen = prefs_touched_at();
    ticker.start(
        slint::TimerMode::Repeated,
        Duration::from_millis(80),
        move || {
            ticks += 1;
            if ticks.is_multiple_of(25) {
                let now = prefs_touched_at();
                if now != prefs_seen {
                    prefs_seen = now;
                    let prefs = store().prefs();
                    words.set(Language::chosen(prefs.locale).strings());
                    if let Some(tray) = tray.as_ref() {
                        tray.show(!prefs.hide_tray);
                        tray.relabel(&words.get());
                    }
                }
            }
            if ticks.is_multiple_of(12)
                && let Some(window) = notice_handle.upgrade()
                && window.window().is_visible()
            {
                let left = window.get_left() - 1;
                window.set_left(left);
                if left <= 0 {
                    let _ = window.hide();
                }
            }
            if let Some(tray) = tray.as_ref() {
                tray.drain(&asks);
            }
            while let Ok(asked) = tray_inbox.try_recv() {
                match asked {
                    Asked::Settings => open_settings(),
                    Asked::Quit => slint::quit_event_loop().unwrap_or(()),
                }
            }
            while let Ok(url) = inbox.try_recv() {
                if let Some(fired) = answered_by_rule(&url, host::clicked_in().as_deref()) {
                    if store().prefs().notify_on_rule
                        && let Some(window) = notice_handle.upgrade()
                    {
                        firing.replace(Some(fired.rule_id.clone()));
                        flash(&window, &words.get(), &fired);
                    }
                    continue;
                }
                if let Some(window) = handle.upgrade() {
                    present(&window, &words.get(), &shown, url);
                }
            }
        },
    );

    slint::run_event_loop_until_quit()
}

#[cfg(test)]
mod tests {
    use super::link_from;
    use linkunbound_shell::Reaches;

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

    #[test]
    fn a_link_split_across_arguments_is_put_back_together() {
        let whole = "https://intranet.corp/x?f=activo urgente";
        assert_eq!(
            link_from(&args(&["https://intranet.corp/x?f=activo", "urgente"])).as_deref(),
            Some(whole)
        );
    }

    #[test]
    fn remembering_the_whole_site_for_an_ip_never_captures_a_different_machine() {
        let scope = Reaches::Site
            .scope("https://192.168.1.50/admin", "192.168.1.50")
            .unwrap();
        assert!(scope.matches("https://192.168.1.50/admin", "192.168.1.50"));
        assert!(!scope.matches("https://10.0.1.50/admin", "10.0.1.50"));
    }
}
