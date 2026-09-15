use std::ptr::NonNull;
use std::sync::Mutex;

use block2::RcBlock;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_app_kit::{
    NSAppearance, NSAppearanceCustomization, NSAppearanceNameAqua, NSAppearanceNameDarkAqua,
    NSApplication, NSApplicationActivationPolicy, NSColor, NSEvent, NSEventModifierFlags,
    NSFloatingWindowLevel, NSPasteboard, NSPasteboardTypeString, NSRunningApplication, NSScreen,
    NSView, NSWindow, NSWindowCollectionBehavior, NSWorkspace,
    NSWorkspaceDidActivateApplicationNotification,
};
use objc2_foundation::{
    MainThreadMarker, NSNotification, NSObjectProtocol, NSPoint, NSString, NSUserDefaults,
};

/// AppKit measures upwards from the bottom of the primary screen; the picker is
/// placed downwards from the top. A point is a rectangle of no height.
fn down(up: f64, height: f64, primary: f64) -> f64 {
    primary - up - height
}

fn physical(points: f64, scale: f64) -> i32 {
    (points * scale).round() as i32
}

/// One factor for the whole space rather than each screen's own, so a point
/// converted out and back lands where it started. Mixed scale factors across
/// monitors are the price.
fn primary(mtm: MainThreadMarker) -> Option<(f64, f64)> {
    let screen = NSScreen::screens(mtm).iter().next()?;
    let frame = screen.frame();
    Some((frame.size.height, screen.backingScaleFactor()))
}

fn contains(frame: objc2_foundation::NSRect, at: NSPoint) -> bool {
    at.x >= frame.origin.x
        && at.x < frame.origin.x + frame.size.width
        && at.y >= frame.origin.y
        && at.y < frame.origin.y + frame.size.height
}

#[must_use]
pub fn cursor() -> Option<(i32, i32)> {
    let mtm = MainThreadMarker::new()?;
    let (height, scale) = primary(mtm)?;
    let at = NSEvent::mouseLocation();
    Some((
        physical(at.x, scale),
        physical(down(at.y, 0.0, height), scale),
    ))
}

#[must_use]
pub fn work_area_at(x: i32, y: i32) -> Option<(i32, i32, i32, i32)> {
    let mtm = MainThreadMarker::new()?;
    let (height, scale) = primary(mtm)?;
    let at = NSPoint::new(
        f64::from(x) / scale,
        down(f64::from(y) / scale, 0.0, height),
    );

    let screens = NSScreen::screens(mtm);
    let screen = screens
        .iter()
        .find(|screen| contains(screen.frame(), at))
        .or_else(|| screens.iter().next())?;

    // Not `frame`: the menu bar and the Dock are not room the picker may take.
    let usable = screen.visibleFrame();
    Some((
        physical(usable.origin.x, scale),
        physical(down(usable.origin.y, usable.size.height, height), scale),
        physical(usable.size.width, scale),
        physical(usable.size.height, scale),
    ))
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

pub fn retire(executable: &str) -> usize {
    let Some(mine) = crate::registration::own_bundle_id() else {
        return 0;
    };
    let me = std::process::id();
    NSRunningApplication::runningApplicationsWithBundleIdentifier(&NSString::from_str(&mine))
        .iter()
        .filter(|app| u32::try_from(app.processIdentifier()).ok() != Some(me))
        .filter(|app| {
            app.executableURL()
                .and_then(|url| url.lastPathComponent())
                .is_some_and(|name| name.to_string() == executable)
        })
        .filter(|app| app.terminate())
        .count()
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

#[must_use]
pub fn source_app() -> Option<String> {
    let workspace = NSWorkspace::sharedWorkspace();
    workspace
        .frontmostApplication()
        .as_deref()
        .and_then(named_unless_ours)
        .or_else(|| LAST_SEEN.lock().ok().and_then(|seen| seen.clone()))
        .or_else(|| {
            workspace
                .menuBarOwningApplication()
                .as_deref()
                .and_then(named_unless_ours)
        })
}

#[allow(unsafe_code)]
pub fn copy_text(text: &str) -> bool {
    let board = NSPasteboard::generalPasteboard();
    board.clearContents();
    // SAFETY: a constant the framework defines and never changes.
    let plain = unsafe { NSPasteboardTypeString };
    board.setString_forType(&NSString::from_str(text), plain)
}

fn dark() -> bool {
    NSUserDefaults::standardUserDefaults()
        .stringForKey(&NSString::from_str("AppleInterfaceStyle"))
        .is_some_and(|style| style.to_string().eq_ignore_ascii_case("Dark"))
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
    use super::{contains, down, physical};
    use objc2_foundation::{NSPoint, NSRect, NSSize};

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

    /// The screen is measured in the pixels it is made of and AppKit answers in points: at 2x
    /// the untouched number asks for half the room the picker needs.
    #[test]
    fn points_become_the_pixels_the_screen_is_made_of() {
        assert_eq!(physical(100.0, 1.0), 100);
        assert_eq!(physical(100.0, 2.0), 200);
        assert_eq!(physical(100.4, 1.0), 100, "rounded, never truncated");
        assert_eq!(physical(100.5, 1.0), 101);
    }

    /// A pointer on the far edge belongs to the next screen, not this one, or a picker opened
    /// on a second monitor is placed against the bounds of the first.
    #[test]
    fn a_screen_holds_its_near_edge_and_leaves_the_far_one_to_the_next() {
        let screen = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(1920.0, 1080.0));
        assert!(contains(screen, NSPoint::new(0.0, 0.0)));
        assert!(contains(screen, NSPoint::new(1919.0, 1079.0)));
        assert!(!contains(screen, NSPoint::new(1920.0, 500.0)));
        assert!(!contains(screen, NSPoint::new(-1.0, 500.0)));
    }
}
