#![windows_subsystem = "windows"]

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::mpsc::channel;
use std::time::Duration;

use linkunbound_core::{Language, Rule, Store, Strings, Target, host_of, normalise};
#[cfg(windows)]
use linkunbound_shell::ICON_SIDE;
use linkunbound_shell::tray::{Asked, Tray};
use linkunbound_shell::{Listed, Notice, Picker, Reaches, dress, paint, place, single};
use slint::{ComponentHandle, Model};

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

    pub fn light_windows() -> bool {
        linkunbound_win::windows_are_light()
    }

    pub fn cursor() -> Option<(i32, i32)> {
        linkunbound_win::cursor()
    }

    pub fn work_area_at(x: i32, y: i32) -> Option<(i32, i32, i32, i32)> {
        linkunbound_win::work_area_at(x, y)
    }

    pub fn keep_off_the_taskbar(window: isize) {
        linkunbound_win::keep_off_the_taskbar(window);
    }

    pub fn take_the_keyboard(window: isize) {
        linkunbound_win::take_the_keyboard(window);
    }

    pub fn is_in_front(window: isize) -> bool {
        linkunbound_win::is_in_front(window)
    }

    pub fn copy_text(text: &str) -> bool {
        linkunbound_win::copy_text(text)
    }

    pub fn digit_behind(typed: char) -> Option<u32> {
        linkunbound_win::digit_behind(typed)
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

    pub fn light_windows() -> bool {
        false
    }

    pub fn cursor() -> Option<(i32, i32)> {
        None
    }

    pub fn work_area_at(_x: i32, _y: i32) -> Option<(i32, i32, i32, i32)> {
        None
    }

    pub fn keep_off_the_taskbar(_window: isize) {}

    pub fn take_the_keyboard(_window: isize) {}

    pub fn is_in_front(_window: isize) -> bool {
        true
    }

    pub fn copy_text(_text: &str) -> bool {
        false
    }

    pub fn digit_behind(typed: char) -> Option<u32> {
        typed.to_digit(10).filter(|d| (1..=9).contains(d))
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
    let rule = rules.resolve(url, &host, source)?;
    let browsers = catalogue();
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
    if let Some(handle) = native_handle(notice.window()) {
        host::keep_off_the_taskbar(handle);
    }
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
    let _ = store().edit_rules(|rules| rules.remove(rule_id));
}

/// Reports rather than swallows: the window says the choice was remembered, and
/// a failed write would leave that claim false with nothing on screen to correct
/// it.
fn remember(
    url: &str,
    chosen: &Listed,
    private: bool,
    reach: Reaches,
    source_app: Option<String>,
) -> bool {
    let Some(rule) = rule_for(url, chosen, private, reach, source_app) else {
        return true;
    };
    store()
        .edit_rules(|rules| {
            rules.upsert(rule.clone());
            true
        })
        .is_ok()
}

/// `None` when there is nothing to remember. Only the origin reach carries the
/// app: the others answer the same link whichever one produced it, and without
/// a known origin that reach has nothing to bind to.
fn rule_for(
    url: &str,
    chosen: &Listed,
    private: bool,
    reach: Reaches,
    source_app: Option<String>,
) -> Option<Rule> {
    let host = host_of(url)?;
    let scope = reach.scope(url, &host)?;
    if reach.binds_to_the_source() && source_app.is_none() {
        return None;
    }
    Some(Rule {
        id: String::new(),
        scope,
        source_app: reach.binds_to_the_source().then_some(source_app).flatten(),
        target: Target {
            browser_id: chosen.browser_id.clone(),
            profile_id: chosen.profile_id.clone(),
        },
        private,
    })
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
    /// Read when the link arrived, not when the user picks: by then the picker
    /// itself is the foreground window, and the rule would bind to us.
    source: Option<String>,
    /// Links that arrived while one was already on screen. Redressing the window
    /// under the user would open the wrong one, and dropping them would lose a
    /// click they already made.
    waiting: std::collections::VecDeque<String>,
}

/// A link that arrives while one is already on screen waits its turn. Redressing
/// the window under the user opens the wrong one — they aimed at what they could
/// see — and discarding it loses a click they already made.
///
/// Kept apart from the window so the decision can be checked without one.
fn claims_the_window(shown: &mut Shown, url: String, occupied: bool) -> Option<String> {
    if occupied {
        shown.waiting.push_back(url);
        return None;
    }
    Some(url)
}

fn present(picker: &Picker, words: &Strings, shown: &Rc<RefCell<Shown>>, url: String) {
    let occupied = picker.window().is_visible();
    let Some(url) = claims_the_window(&mut shown.borrow_mut(), url, occupied) else {
        return;
    };
    let listed = rows();
    if listed.is_empty() {
        // Opening settings and dropping the link loses the click: the address is
        // still on screen here, and settings is one press away.
        dress(picker, words, &url, host::clicked_in().as_deref(), &listed);
        picker.set_alarming(false);
        picker.set_problem(words.no_browsers.into());
        shown.borrow_mut().url = Some(url);
        beside_the_pointer(picker);
        let _ = picker.show();
        if let Some(handle) = native_handle(picker.window()) {
            host::keep_off_the_taskbar(handle);
            host::take_the_keyboard(handle);
        }
        return;
    }
    let source = host::clicked_in();
    dress(picker, words, &url, source.as_deref(), &listed);
    {
        let mut held = shown.borrow_mut();
        held.url = Some(url);
        held.rows = listed;
        held.source = source;
    }
    beside_the_pointer(picker);
    let _ = picker.show();
    if let Some(handle) = native_handle(picker.window()) {
        host::keep_off_the_taskbar(handle);
        host::take_the_keyboard(handle);
    }
}

/// The picker is summoned by a click and answered with the keyboard, so it has
/// to be in front and hold the focus; winit hands over neither on its own.
fn native_handle(window: &slint::Window) -> Option<isize> {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    match window.window_handle().window_handle().ok()?.as_raw() {
        RawWindowHandle::Win32(win32) => Some(win32.hwnd.get()),
        _ => None,
    }
}

/// Shows whatever queued up behind the link just dealt with, so nothing a user
/// clicked is silently dropped.
fn next_in_line(picker: &Picker, words: &Strings, shown: &Rc<RefCell<Shown>>) {
    let queued = shown.borrow_mut().waiting.pop_front();
    if let Some(url) = queued {
        present(picker, words, shown, url);
        if let Some(ui) = ui() {
            // The next link needs its own settle: carrying the previous one's
            // state would dismiss it on the tick after it appeared.
            ui.watch_focus();
        }
    }
}

/// The picker belongs to the click that summoned it, not to the middle of a screen.
fn beside_the_pointer(picker: &Picker) {
    let Some((cx, cy)) = host::cursor() else {
        return;
    };
    let Some((x, y, width, height)) = host::work_area_at(cx, cy) else {
        return;
    };
    let scale = f64::from(picker.window().scale_factor());
    let size = |logical: f32| (f64::from(logical) * scale).round() as i32;

    let (at_x, at_y) = place::beside(
        (cx, cy),
        (
            size(picker.get_wanted_width()),
            size(picker.get_wanted_height()),
        ),
        place::Bounds {
            x,
            y,
            width,
            height,
        },
    );
    picker
        .window()
        .set_position(slint::PhysicalPosition::new(at_x, at_y));
}

/// Everything the event loop owns. Reached from the listener thread by way of
/// `invoke_from_event_loop`, which is why it lives here and not in a closure:
/// the state is `Rc` and cannot cross a thread, but a wake-up can.
struct Ui {
    picker: Picker,
    notice: Notice,
    shown: Rc<RefCell<Shown>>,
    words: Cell<Strings>,
    firing: RefCell<Option<String>>,
    tray: Option<Tray>,
    watch: slint::Timer,
    countdown: slint::Timer,
    prefs_seen: Cell<Option<std::time::SystemTime>>,
    held_focus: Cell<bool>,
}

thread_local! {
    static UI: RefCell<Option<Rc<Ui>>> = const { RefCell::new(None) };
}

fn ui() -> Option<Rc<Ui>> {
    UI.with_borrow(Clone::clone)
}

impl Ui {
    /// Settings runs in another process: the file is the only channel between
    /// them. Read when something is about to be shown rather than on a clock.
    fn catch_up(&self) {
        let now = prefs_touched_at();
        if now == self.prefs_seen.get() {
            return;
        }
        self.prefs_seen.set(now);
        self.obey(&store().prefs());
    }

    fn obey(&self, prefs: &linkunbound_core::Preferences) {
        self.words.set(Language::chosen(prefs.locale).strings());
        paint(&self.picker, &self.notice, wants_light(prefs.theme));
        if let Some(tray) = self.tray.as_ref() {
            tray.show(!prefs.hide_tray);
            tray.relabel(&self.words.get());
        }
    }

    /// Slint offers no focus-lost event, so this polls — but only while the
    /// picker is on screen, and it stops itself the moment it is not.
    fn watch_focus(self: &Rc<Self>) {
        self.held_focus.set(false);
        let weak = Rc::downgrade(self);
        self.watch.start(
            slint::TimerMode::Repeated,
            Duration::from_millis(60),
            move || {
                let Some(ui) = weak.upgrade() else { return };
                if !ui.picker.window().is_visible() {
                    ui.watch.stop();
                    return;
                }
                // Without a handle nothing is known yet, and taking that for
                // "in front" armed the dismissal before the window ever had the
                // focus: the picker vanished on the tick after it appeared.
                let Some(ours) = native_handle(ui.picker.window()) else {
                    return;
                };
                if host::is_in_front(ours) {
                    ui.held_focus.set(true);
                } else if ui.held_focus.get() {
                    let _ = ui.picker.hide();
                    ui.watch.stop();
                    // Whatever queued behind this link is still a click the user
                    // made; dropping it here loses it without a word.
                    next_in_line(&ui.picker, &ui.words.get(), &ui.shown);
                }
            },
        );
    }

    fn count_down(self: &Rc<Self>) {
        let weak = Rc::downgrade(self);
        self.countdown.start(
            slint::TimerMode::Repeated,
            Duration::from_secs(1),
            move || {
                let Some(ui) = weak.upgrade() else { return };
                let left = ui.notice.get_left() - 1;
                ui.notice.set_left(left);
                if left <= 0 {
                    let _ = ui.notice.hide();
                    ui.countdown.stop();
                }
            },
        );
    }
}

/// One link, start to finish, on the UI thread. Called the instant the socket
/// reads it: waiting for a poll turned four milliseconds of work into sixty.
fn arrived(raw: String) {
    let Some(ui) = ui() else { return };
    // The socket takes a line from any process of this user; the command line is
    // normalised and this has to be too, or the scheme guard is walked around.
    let Some(url) = normalise(&raw) else { return };
    ui.catch_up();

    if let Some(fired) = answered_by_rule(&url, host::clicked_in().as_deref()) {
        if store().prefs().notify_on_rule {
            ui.firing.replace(Some(fired.rule_id.clone()));
            flash(&ui.notice, &ui.words.get(), &fired);
            ui.count_down();
        }
        return;
    }
    present(&ui.picker, &ui.words.get(), &ui.shown, url);
    ui.watch_focus();
}

fn wants_light(theme: linkunbound_core::Theme) -> bool {
    match theme {
        linkunbound_core::Theme::Light => true,
        linkunbound_core::Theme::Dark => false,
        linkunbound_core::Theme::System => host::light_windows(),
    }
}

fn asked_for(what: Asked) {
    match what {
        Asked::Settings => open_settings(),
        Asked::Quit => slint::quit_event_loop().unwrap_or(()),
    }
}

fn main() -> Result<(), slint::PlatformError> {
    let args: Vec<String> = std::env::args_os()
        .map(|a| a.to_string_lossy().into_owned())
        .collect();
    let incoming = link_from(&args);

    let Some(_server) = single::claim(|url| {
        let _ = slint::invoke_from_event_loop(move || arrived(url));
    }) else {
        if let Some(url) = incoming {
            single::hand_over(&url);
        } else {
            open_settings();
        }
        return Ok(());
    };

    let picker = Picker::new()?;
    let shown = Rc::new(RefCell::new(Shown::default()));

    {
        let handle = picker.as_weak();
        let shown = Rc::clone(&shown);
        picker.on_dismissed(move || {
            let Some(window) = handle.upgrade() else {
                return;
            };
            let _ = window.hide();
            if let Some(ui) = ui() {
                next_in_line(&window, &ui.words.get(), &shown);
            }
        });
    }
    {
        let handle = picker.as_weak();
        picker.on_typed(move |typed, private| {
            let Some(window) = handle.upgrade() else {
                return false;
            };
            let Some(digit) = typed.chars().next().and_then(host::digit_behind) else {
                return false;
            };
            let Some(index) = i32::try_from(digit).ok().map(|d| d - 1) else {
                return false;
            };
            if index >= window.get_rows().row_count().try_into().unwrap_or(i32::MAX) {
                return false;
            }
            window.invoke_open(index, private);
            true
        });
    }
    {
        let handle = picker.as_weak();
        let shown = Rc::clone(&shown);
        picker.on_open(move |index, private| {
            let Some(window) = handle.upgrade() else {
                return;
            };
            let reach = Reaches::at(window.get_reach_index());
            let held = shown.borrow();
            let Some(url) = held.url.clone() else { return };
            let source = held.source.clone();
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
                    let words = ui().map(|ui| ui.words.get());
                    if remember(&url, chosen, private, reach, source) {
                        let _ = window.hide();
                        drop(held);
                        if let Some(words) = words {
                            next_in_line(&window, &words, &shown);
                        }
                    } else if let Some(words) = words {
                        window.set_alarming(true);
                        window.set_problem(words.fail_not_remembered.into());
                    }
                }
                Err(why) => {
                    if let Some(ui) = ui() {
                        window.set_alarming(true);
                        window.set_problem(ui.words.get().on_failure(&why).into());
                    }
                }
            }
        });
    }
    {
        let handle = picker.as_weak();
        let shown = Rc::clone(&shown);
        picker.on_copy(move || {
            let Some(window) = handle.upgrade() else {
                return;
            };
            let url = shown.borrow().url.clone();
            // The tick only ever animated: nothing had reached the clipboard.
            if let Some(url) = url {
                window.set_copied(host::copy_text(&url));
            }
        });
    }

    let notice = Notice::new()?;
    {
        let handle = notice.as_weak();
        notice.on_undo(move || {
            if let Some(id) = ui().and_then(|ui| ui.firing.borrow_mut().take()) {
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

    let spoken = Language::chosen(store().prefs().locale).strings();
    let tray = Tray::install(host::light_taskbar(), &spoken);
    let (asks, tray_inbox) = channel::<Asked>();

    let state = Rc::new(Ui {
        picker,
        notice,
        shown,
        words: Cell::new(spoken),
        firing: RefCell::new(None),
        tray,
        watch: slint::Timer::default(),
        countdown: slint::Timer::default(),
        prefs_seen: Cell::new(prefs_touched_at()),
        held_focus: Cell::new(false),
    });
    // Applied here rather than only on a later change: the tray was built
    // visible and the theme never left settings at all.
    state.obey(&store().prefs());
    UI.with_borrow_mut(|slot| *slot = Some(Rc::clone(&state)));

    // Deferred into the loop rather than presented here: before it runs there is no window
    // handle, so the picker would open with a taskbar button and without the keyboard.
    if let Some(url) = incoming {
        let _ = slint::invoke_from_event_loop(move || arrived(url));
    }

    // The tray hands its events to a global queue rather than a callback, so
    // this is the one thing still on a clock — and only while the app is idle,
    // which is exactly when nothing else needs the CPU.
    let pump = slint::Timer::default();
    pump.start(
        slint::TimerMode::Repeated,
        Duration::from_millis(120),
        move || {
            if let Some(tray) = state.tray.as_ref() {
                tray.drain(&asks);
            }
            while let Ok(what) = tray_inbox.try_recv() {
                if what == Asked::Settings {
                    state.catch_up();
                }
                asked_for(what);
            }
        },
    );

    slint::run_event_loop_until_quit()
}

#[cfg(test)]
mod tests {
    use super::{Listed, Shown, claims_the_window, link_from, rule_for};
    use linkunbound_core::Scope;
    use linkunbound_core::normalise;
    use linkunbound_shell::Reaches;

    fn args(rest: &[&str]) -> Vec<String> {
        std::iter::once("linkunbound.exe")
            .chain(rest.iter().copied())
            .map(str::to_owned)
            .collect()
    }

    /// The socket accepts a line from any process of this user, so what it
    /// carries is as untrusted as a command line and has to cross the same
    /// guard. A `file://` reaching a rule would fetch a UNC path and hand over
    /// the NTLM hash without a click.
    #[test]
    fn what_arrives_over_the_socket_meets_the_same_guard_as_the_command_line() {
        for hostile in [
            "file://evil.test/share/payload",
            "--gpu-launcher=calc.exe",
            "javascript:alert(1)",
            "",
        ] {
            assert!(
                normalise(hostile).is_none(),
                "{hostile} must not survive the guard"
            );
        }
        assert_eq!(
            normalise("https://github.com/a").as_deref(),
            Some("https://github.com/a")
        );
    }

    /// A link arriving while one is on screen must not redress the window: the
    /// user aimed at what they could see. And it must not be dropped either —
    /// that click already happened.
    #[test]
    fn a_link_arriving_over_a_shown_one_waits_instead_of_replacing_it() {
        let mut shown = Shown::default();

        let first = claims_the_window(&mut shown, "https://first.test/a".to_owned(), false);
        assert_eq!(
            first.as_deref(),
            Some("https://first.test/a"),
            "with no window up, the link is dressed straight away"
        );
        assert!(shown.waiting.is_empty());

        let second = claims_the_window(&mut shown, "https://second.test/b".to_owned(), true);
        assert!(second.is_none(), "it must not take a window already in use");
        let third = claims_the_window(&mut shown, "https://third.test/c".to_owned(), true);
        assert!(third.is_none());

        assert_eq!(
            shown.waiting.pop_front().as_deref(),
            Some("https://second.test/b"),
            "queued links are answered in the order they were clicked"
        );
        assert_eq!(shown.waiting.len(), 1, "and none of them is dropped");
    }

    fn chosen() -> Listed {
        Listed {
            browser_id: "chrome".to_owned(),
            profile_id: Some("Default".to_owned()),
            name: "Google Chrome".to_owned(),
            profile: "Personal".to_owned(),
            icon: None,
            can_private: true,
        }
    }

    /// The origin is read when the link arrives, because by the time the user
    /// picks, the picker itself is the foreground window.
    #[test]
    fn only_the_origin_reach_binds_the_rule_to_an_app() {
        let url = "https://github.com/a";

        let bound = rule_for(
            url,
            &chosen(),
            false,
            Reaches::FromApp,
            Some("teams".to_owned()),
        )
        .expect("an origin reach with a known origin makes a rule");
        assert_eq!(bound.source_app.as_deref(), Some("teams"));
        assert_eq!(bound.scope, Scope::Any, "it covers any address");

        let site = rule_for(
            url,
            &chosen(),
            false,
            Reaches::Site,
            Some("teams".to_owned()),
        )
        .expect("a site reach makes a rule too");
        assert_eq!(
            site.source_app, None,
            "but it answers whichever app produced the link"
        );
        assert_eq!(site.scope, Scope::Site("github.com".to_owned()));
    }

    #[test]
    fn an_origin_rule_is_not_written_without_an_origin() {
        assert!(
            rule_for(
                "https://github.com/a",
                &chosen(),
                false,
                Reaches::FromApp,
                None
            )
            .is_none()
        );
    }

    #[test]
    fn just_this_time_leaves_nothing_behind() {
        assert!(
            rule_for(
                "https://github.com/a",
                &chosen(),
                false,
                Reaches::Once,
                None
            )
            .is_none()
        );
    }

    #[test]
    fn the_rule_carries_the_profile_and_the_private_choice() {
        let rule = rule_for("https://github.com/a", &chosen(), true, Reaches::Site, None)
            .expect("should make a rule");
        assert_eq!(rule.target.browser_id, "chrome");
        assert_eq!(rule.target.profile_id.as_deref(), Some("Default"));
        assert!(rule.private);
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
