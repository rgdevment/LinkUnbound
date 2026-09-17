use std::ptr::NonNull;
use std::sync::Mutex;

use block2::RcBlock;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_app_kit::{
    NSAppearance, NSAppearanceCustomization, NSAppearanceNameAqua, NSAppearanceNameDarkAqua,
    NSApplication, NSApplicationActivationPolicy, NSColor, NSEvent, NSEventModifierFlags,
    NSEventType, NSFloatingWindowLevel, NSPasteboard, NSPasteboardTypeString, NSRunningApplication,
    NSScreen, NSView, NSWindow, NSWindowCollectionBehavior, NSWorkspace,
    NSWorkspaceDidActivateApplicationNotification,
};
use objc2_foundation::{
    MainThreadMarker, NSNotification, NSObjectProtocol, NSPoint, NSRect, NSString, NSUserDefaults,
};

/// AppKit measures upwards from the bottom of the primary screen; the picker is
/// placed downwards from the top. A point is a rectangle of no height.
fn down(up: f64, height: f64, primary: f64) -> f64 {
    primary - up - height
}

fn whole(points: f64) -> i32 {
    points.round() as i32
}

/// Everything here stays in points. The screen space is one coordinate system
/// in points whatever each screen's scale, while a pixel count only means
/// something for the screen it was taken on: a window that last sat on the
/// other screen would divide by the wrong factor.
fn primary_height(mtm: MainThreadMarker) -> Option<f64> {
    Some(NSScreen::screens(mtm).iter().next()?.frame().size.height)
}

fn contains(frame: NSRect, at: NSPoint) -> bool {
    at.x >= frame.origin.x
        && at.x < frame.origin.x + frame.size.width
        && at.y >= frame.origin.y
        && at.y < frame.origin.y + frame.size.height
}

fn located(at: NSPoint, height: f64) -> (i32, i32) {
    (whole(at.x), whole(down(at.y, 0.0, height)))
}

fn pointed(x: i32, y: i32, height: f64) -> NSPoint {
    NSPoint::new(f64::from(x), down(f64::from(y), 0.0, height))
}

fn placed(usable: NSRect, height: f64) -> (i32, i32, i32, i32) {
    (
        whole(usable.origin.x),
        whole(down(usable.origin.y, usable.size.height, height)),
        whole(usable.size.width),
        whole(usable.size.height),
    )
}

#[must_use]
pub fn cursor() -> Option<(i32, i32)> {
    let mtm = MainThreadMarker::new()?;
    let height = primary_height(mtm)?;
    Some(located(NSEvent::mouseLocation(), height))
}

#[must_use]
pub fn work_area_at(x: i32, y: i32) -> Option<(i32, i32, i32, i32)> {
    let mtm = MainThreadMarker::new()?;
    let height = primary_height(mtm)?;
    let at = pointed(x, y, height);

    let screens = NSScreen::screens(mtm);
    let screen = screens
        .iter()
        .find(|screen| contains(screen.frame(), at))
        .or_else(|| screens.iter().next())?;

    // Not `frame`: the menu bar and the Dock are not room the picker may take.
    Some(placed(screen.visibleFrame(), height))
}

#[allow(unsafe_code)]
fn window_of(view: isize) -> Option<Retained<NSWindow>> {
    let view = view as *const NSView;
    if view.is_null() {
        return None;
    }
    unsafe { &*view }.window()
}

pub fn keep_off_the_taskbar(view: isize, corner: f64) {
    let Some(window) = window_of(view) else {
        return;
    };
    window.setLevel(NSFloatingWindowLevel);
    window.setCollectionBehavior(
        NSWindowCollectionBehavior::CanJoinAllSpaces
            | NSWindowCollectionBehavior::FullScreenAuxiliary,
    );
    window.setHidesOnDeactivate(false);
    window.setOpaque(false);
    window.setBackgroundColor(Some(&NSColor::clearColor()));
    window.setHasShadow(true);
    if let Some(content) = window.contentView() {
        content.setWantsLayer(true);
        if let Some(layer) = content.layer() {
            layer.setCornerRadius(corner);
            layer.setMasksToBounds(true);
        }
    }
    window.invalidateShadow();
}

pub fn take_the_keyboard(view: isize) {
    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };
    let Some(window) = window_of(view) else {
        return;
    };
    let app = NSApplication::sharedApplication(mtm);
    app.setActivationPolicy(NSApplicationActivationPolicy::Regular);
    #[allow(deprecated)]
    app.activateIgnoringOtherApps(true);
    window.makeKeyAndOrderFront(None);
}

#[must_use]
pub fn is_in_front(view: isize) -> bool {
    window_of(view).is_some_and(|window| window.isKeyWindow())
}

/// Whether this application is the active one: what is in front is then a window of its own.
#[must_use]
pub fn front_is_ours() -> bool {
    MainThreadMarker::new().is_some_and(|mtm| NSApplication::sharedApplication(mtm).isActive())
}

/// The application in front, by process: the one the picker was shown over, so that the person
/// leaving it can be told from the picker simply not being key.
#[must_use]
pub fn front_application() -> isize {
    NSWorkspace::sharedWorkspace()
        .frontmostApplication()
        .map_or(0, |app| app.processIdentifier() as isize)
}

pub fn let_whoever_opens_next_come_forward() {
    if let Some(mtm) = MainThreadMarker::new() {
        let app = NSApplication::sharedApplication(mtm);
        app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);
        app.deactivate();
    }
}

#[allow(unsafe_code)]
pub fn dress_window(window: isize, dark: Option<bool>) {
    let window = window as *const NSWindow;
    if window.is_null() {
        return;
    }
    let appearance = dark.and_then(|dark| {
        let name = unsafe {
            if dark {
                NSAppearanceNameDarkAqua
            } else {
                NSAppearanceNameAqua
            }
        };
        NSAppearance::appearanceNamed(name)
    });
    unsafe { &*window }.setAppearance(appearance.as_deref());
}

fn is_another_copy_of(pid: i32, me: u32, executable: Option<&str>, wanted: &str) -> bool {
    u32::try_from(pid).ok() != Some(me) && executable == Some(wanted)
}

fn other_copies_of(executable: &str) -> Vec<Retained<NSRunningApplication>> {
    let Some(mine) = crate::registration::own_bundle_id() else {
        return Vec::new();
    };
    let me = std::process::id();
    NSRunningApplication::runningApplicationsWithBundleIdentifier(&NSString::from_str(&mine))
        .iter()
        .filter(|app| {
            let name = app
                .executableURL()
                .and_then(|url| url.lastPathComponent())
                .map(|name| name.to_string());
            is_another_copy_of(app.processIdentifier(), me, name.as_deref(), executable)
        })
        .collect()
}

/// `terminate` only asks. The copy that replaces these needs the socket they
/// hold, so this waits for them to be gone, and stops asking after a while.
#[allow(unsafe_code)]
pub fn retire(executable: &str) -> usize {
    objc2::rc::autoreleasepool(|_| {
        let copies = other_copies_of(executable);
        let asked = copies.len();
        for app in &copies {
            let _ = app.terminate();
        }
        let patience = std::time::Instant::now() + LONG_ENOUGH_TO_QUIT;
        while !other_copies_of(executable).is_empty() && std::time::Instant::now() < patience {
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        // Whoever is still standing, asked or not: a copy that refused the request kept the
        // socket, and the one taking over then opened a settings window instead of the picker.
        for app in other_copies_of(executable) {
            if !app.forceTerminate() {
                // SAFETY: `kill` takes a pid and a signal and touches no memory of ours; the pid
                // was read a moment ago from the running application it names.
                unsafe {
                    libc::kill(app.processIdentifier(), libc::SIGKILL);
                }
            }
        }
        asked
    })
}

const LONG_ENOUGH_TO_QUIT: std::time::Duration = std::time::Duration::from_secs(5);

/// The hardware key rather than the character it produced: with Shift held for
/// a private window the top row reads `!@#`, and on a French layout it reads
/// `&é"` with nothing held at all. Windows reads the same key back through
/// `VkKeyScanW`.
const DIGIT_ROW: [u16; 9] = [0x12, 0x13, 0x14, 0x15, 0x17, 0x16, 0x1A, 0x1C, 0x19];
const KEYPAD: [u16; 9] = [0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5B, 0x5C];

fn digit_at(key_code: u16) -> Option<u32> {
    DIGIT_ROW
        .iter()
        .chain(KEYPAD.iter())
        .position(|code| *code == key_code)
        .map(|index| u32::try_from(index % 9 + 1).unwrap_or(0))
}

fn typed_digit(typed: char) -> Option<u32> {
    typed.to_digit(10).filter(|d| (1..=9).contains(d))
}

#[must_use]
pub fn digit_behind(typed: char) -> Option<u32> {
    let pressed = MainThreadMarker::new()
        .and_then(|mtm| NSApplication::sharedApplication(mtm).currentEvent())
        .filter(|event| event.r#type() == NSEventType::KeyDown)
        .map(|event| event.keyCode());
    match pressed {
        Some(code) => digit_at(code),
        None => typed_digit(typed),
    }
}

#[must_use]
pub fn shift_is_down() -> bool {
    NSEvent::modifierFlags_class().contains(NSEventModifierFlags::Shift)
}

fn named_unless_ours(app: &NSRunningApplication) -> Option<String> {
    let bundle_id = app.bundleIdentifier()?.to_string();
    if crate::is_one_of_ours(&bundle_id) {
        return None;
    }
    let name = app.localizedName()?.to_string().to_lowercase();
    (!name.is_empty()).then_some(name)
}

static LAST_SEEN: Mutex<Option<String>> = Mutex::new(None);

fn note_activation(app: Option<Retained<NSRunningApplication>>) {
    if let Some(name) = app.as_deref().and_then(named_unless_ours)
        && let Ok(mut seen) = LAST_SEEN.lock()
    {
        *seen = Some(name);
    }
}

pub struct Watching(Retained<ProtocolObject<dyn NSObjectProtocol>>);

#[allow(unsafe_code)]
#[must_use]
pub fn watch_activations() -> Watching {
    let block = RcBlock::new(|_: NonNull<NSNotification>| {
        note_activation(NSWorkspace::sharedWorkspace().frontmostApplication());
    });
    let observer = unsafe {
        NSWorkspace::sharedWorkspace()
            .notificationCenter()
            .addObserverForName_object_queue_usingBlock(
                Some(NSWorkspaceDidActivateApplicationNotification),
                None,
                None,
                &block,
            )
    };
    note_activation(NSWorkspace::sharedWorkspace().frontmostApplication());
    Watching(observer)
}

impl Drop for Watching {
    #[allow(unsafe_code)]
    fn drop(&mut self) {
        unsafe {
            NSWorkspace::sharedWorkspace()
                .notificationCenter()
                .removeObserver(self.0.as_ref());
        }
    }
}

fn origin(
    front: Option<String>,
    last_seen: Option<String>,
    menu_bar: Option<String>,
) -> Option<String> {
    front.or(last_seen).or(menu_bar)
}

#[must_use]
pub fn source_app() -> Option<String> {
    let workspace = NSWorkspace::sharedWorkspace();
    origin(
        workspace
            .frontmostApplication()
            .as_deref()
            .and_then(named_unless_ours),
        LAST_SEEN.lock().ok().and_then(|seen| seen.clone()),
        workspace
            .menuBarOwningApplication()
            .as_deref()
            .and_then(named_unless_ours),
    )
}

#[allow(unsafe_code)]
pub fn copy_text(text: &str) -> bool {
    let board = NSPasteboard::generalPasteboard();
    board.clearContents();
    // SAFETY: a constant the framework defines and never changes.
    let plain = unsafe { NSPasteboardTypeString };
    board.setString_forType(&NSString::from_str(text), plain)
}

fn is_dark(style: Option<String>) -> bool {
    style.is_some_and(|style| style.eq_ignore_ascii_case("Dark"))
}

fn dark() -> bool {
    is_dark(
        NSUserDefaults::standardUserDefaults()
            .stringForKey(&NSString::from_str("AppleInterfaceStyle"))
            .map(|style| style.to_string()),
    )
}

#[must_use]
pub fn menu_bar_is_light() -> bool {
    !dark()
}

#[must_use]
pub fn windows_are_light() -> bool {
    !dark()
}

#[cfg(test)]
mod tests {
    use super::{
        LAST_SEEN, contains, down, is_another_copy_of, located, named_unless_ours, note_activation,
        origin, placed, pointed, retire, whole,
    };
    use objc2_app_kit::{NSRunningApplication, NSWorkspace};
    use objc2_foundation::{NSPoint, NSRect, NSSize};

    /// A screen of 1440 points with the menu bar taking the top 25 and the Dock the bottom
    /// 70: the usable rectangle AppKit reports starts 70 up from the floor.
    fn a_laptop_screen() -> (NSRect, f64) {
        let usable = NSRect::new(NSPoint::new(0.0, 70.0), NSSize::new(2560.0, 1345.0));
        (usable, 1440.0)
    }

    #[test]
    fn the_work_area_is_read_from_the_top_in_points() {
        let (usable, height) = a_laptop_screen();
        assert_eq!(placed(usable, height), (0, 25, 2560, 1345));
        let second = NSRect::new(NSPoint::new(2560.0, -200.0), NSSize::new(1920.0, 1000.0));
        assert_eq!(placed(second, height), (2560, 640, 1920, 1000));
        let odd = NSRect::new(NSPoint::new(0.4, 0.6), NSSize::new(100.5, 99.5));
        assert_eq!(placed(odd, 200.0), (0, 100, 101, 100));
    }

    #[test]
    fn a_point_names_the_same_place_in_both_directions() {
        let (_, height) = a_laptop_screen();
        let at = pointed(1000, 300, height);
        assert_eq!((at.x, at.y), (1000.0, 1140.0));
        assert_eq!(located(at, height), (1000, 300));
        assert_eq!(located(NSPoint::new(0.0, height), height), (0, 0));
        assert_eq!(located(NSPoint::new(10.6, 1439.4), height), (11, 1));
    }

    /// Asked from any thread but the main one there is no answer, never a made-up one: a
    /// picker placed from a guess lands on the wrong screen.
    #[test]
    fn the_screen_is_only_read_from_the_main_thread() {
        assert!(super::cursor().is_none());
        assert!(super::work_area_at(0, 0).is_none());
    }

    /// The picker reads the key, not the glyph: Shift is what asks for a private window, and
    /// on a French layout the top row produces no digit without it.
    #[test]
    fn a_digit_is_read_from_the_key_that_carries_it() {
        assert_eq!(super::digit_at(0x12), Some(1));
        assert_eq!(super::digit_at(0x19), Some(9));
        assert_eq!(super::digit_at(0x53), Some(1), "the keypad counts too");
        assert_eq!(super::digit_at(0x5C), Some(9));
        assert_eq!(super::digit_at(0x1D), None, "zero opens nothing");
        assert_eq!(super::digit_at(0x00), None, "a is not a digit");
        assert_eq!(super::typed_digit('7'), Some(7));
        assert_eq!(super::typed_digit('0'), None);
        assert_eq!(super::typed_digit('!'), None);
        assert_eq!(
            super::digit_behind('5'),
            Some(5),
            "without a key event in flight the glyph is all there is"
        );
        assert_eq!(super::digit_behind('!'), None);
    }

    #[test]
    fn a_handle_to_nothing_is_left_alone() {
        super::keep_off_the_taskbar(0, 10.0);
        super::take_the_keyboard(0);
        super::dress_window(0, Some(true));
        assert!(!super::is_in_front(0));
        assert!(super::window_of(0).is_none());
        super::let_whoever_opens_next_come_forward();
    }

    #[test]
    fn the_menu_bar_and_the_windows_follow_the_same_switch() {
        use objc2_foundation::{NSString, NSUserDefaults};
        assert_eq!(super::menu_bar_is_light(), super::windows_are_light());
        assert_eq!(super::menu_bar_is_light(), !super::dark());
        let system_says = NSUserDefaults::standardUserDefaults()
            .stringForKey(&NSString::from_str("AppleInterfaceStyle"))
            .map(|style| style.to_string());
        assert_eq!(
            super::dark(),
            system_says
                .as_deref()
                .is_some_and(|s| s.eq_ignore_ascii_case("dark")),
            "the switch is the system's own setting, {system_says:?}"
        );
        let some = |s: &str| Some(s.to_owned());
        assert!(super::is_dark(some("Dark")));
        assert!(super::is_dark(some("dark")));
        assert!(!super::is_dark(some("Light")));
        assert!(!super::is_dark(some("")));
        assert!(!super::is_dark(None));
    }

    #[test]
    fn an_origin_is_named_in_lower_case_and_never_by_us() {
        let me = NSRunningApplication::currentApplication();
        assert_eq!(named_unless_ours(&me), None, "a test binary has no bundle");
        let running = NSWorkspace::sharedWorkspace().runningApplications();
        let someone = running
            .iter()
            .find(|app| app.bundleIdentifier().is_some() && app.localizedName().is_some())
            .expect("something is always running");
        let name = named_unless_ours(&someone).expect("a name");
        assert_eq!(name, name.to_lowercase());
        assert!(!name.is_empty());

        note_activation(None);
        note_activation(Some(someone.clone()));
        assert_eq!(
            LAST_SEEN.lock().ok().and_then(|seen| seen.clone()),
            Some(name)
        );
        let _watching = super::watch_activations();
        let workspace = NSWorkspace::sharedWorkspace();
        let candidates: Vec<Option<String>> = vec![
            workspace
                .frontmostApplication()
                .as_deref()
                .and_then(named_unless_ours),
            LAST_SEEN.lock().ok().and_then(|seen| seen.clone()),
            workspace
                .menuBarOwningApplication()
                .as_deref()
                .and_then(named_unless_ours),
        ];
        let origin = super::source_app().expect("somebody was activated last");
        assert!(!origin.is_empty());
        assert_eq!(origin, origin.to_lowercase());
        assert!(
            candidates.contains(&Some(origin.clone())),
            "{origin} vs {candidates:?}"
        );
    }

    #[test]
    fn the_origin_is_whoever_was_in_front_then_whoever_was_activated_last_then_the_menu_bar() {
        let some = |s: &str| Some(s.to_owned());
        assert_eq!(
            origin(some("slack"), some("finder"), some("orca")),
            some("slack")
        );
        assert_eq!(origin(None, some("finder"), some("orca")), some("finder"));
        assert_eq!(origin(None, None, some("orca")), some("orca"));
        assert_eq!(origin(None, None, None), None);
    }

    #[test]
    fn only_another_process_running_the_named_program_is_retired() {
        assert!(is_another_copy_of(
            41,
            7,
            Some("linkunbound-shell"),
            "linkunbound-shell"
        ));
        assert!(!is_another_copy_of(
            7,
            7,
            Some("linkunbound-shell"),
            "linkunbound-shell"
        ));
        assert!(!is_another_copy_of(
            41,
            7,
            Some("linkunbound-settings"),
            "linkunbound-shell"
        ));
        assert!(!is_another_copy_of(41, 7, None, "linkunbound-shell"));
        assert_eq!(
            retire("linkunbound-shell"),
            0,
            "a test binary has no bundle to look through"
        );
    }

    /// The two systems disagree about which way is up, and taking one for the other opened
    /// the picker as far from the pointer as the pointer was from the other edge.
    #[test]
    fn a_point_measured_from_the_bottom_is_read_from_the_top() {
        assert_eq!(down(0.0, 0.0, 1000.0), 1000.0, "the floor is the far edge");
        assert_eq!(
            down(1000.0, 0.0, 1000.0),
            0.0,
            "and the ceiling is the near one"
        );
        assert_eq!(down(400.0, 0.0, 1000.0), 600.0);
    }

    /// A rectangle hangs below the point that names it, so its own height has to come off too:
    /// without that the work area started a screen's height further down than it does.
    #[test]
    fn a_rectangle_is_read_from_its_own_top_rather_than_its_bottom() {
        assert_eq!(down(25.0, 900.0, 1000.0), 75.0);
        assert_eq!(
            down(0.0, 1000.0, 1000.0),
            0.0,
            "one filling the screen starts at the top"
        );
    }

    #[test]
    fn a_fraction_of_a_point_is_rounded_never_truncated() {
        assert_eq!(whole(100.0), 100);
        assert_eq!(whole(100.4), 100);
        assert_eq!(whole(100.5), 101);
        assert_eq!(whole(-0.6), -1);
    }

    /// A pointer on the far edge belongs to the next screen, not this one, or a picker opened
    /// on a second monitor is placed against the bounds of the first.
    #[test]
    fn a_screen_holds_its_near_edge_and_leaves_the_far_one_to_the_next() {
        let screen = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(1920.0, 1080.0));
        assert!(contains(screen, NSPoint::new(0.0, 0.0)));
        assert!(contains(screen, NSPoint::new(1919.0, 1079.0)));
        assert!(!contains(screen, NSPoint::new(1920.0, 500.0)));
        assert!(!contains(screen, NSPoint::new(500.0, 1080.0)));
        assert!(!contains(screen, NSPoint::new(-1.0, 500.0)));
        assert!(!contains(screen, NSPoint::new(500.0, -1.0)));
    }
}
