use std::ptr::NonNull;

use block2::RcBlock;
use objc2::Message;
use objc2::rc::Retained;
use objc2::runtime::AnyClass;
use objc2_app_kit::{
    NSAppearance, NSAppearanceCustomization, NSAppearanceNameAqua, NSAppearanceNameDarkAqua,
    NSBackingStoreType, NSColor, NSGlassEffectView, NSUserInterfaceItemIdentification, NSView,
    NSViewFrameDidChangeNotification, NSVisualEffectBlendingMode, NSVisualEffectMaterial,
    NSVisualEffectState, NSVisualEffectView, NSWindow, NSWindowDidMoveNotification,
    NSWindowDidResizeNotification, NSWindowOrderingMode, NSWindowStyleMask, NSWorkspace,
};
use objc2_foundation::{MainThreadMarker, NSNotification, NSNotificationCenter, NSString};

use crate::native::window_of;

const GLASS: &str = "linkunbound-glass";

/// Asked by name: the class only exists from macOS 26, and naming it in code, a downcast
/// included, panics on the systems before it.
fn glass_exists() -> bool {
    AnyClass::get(c"NSGlassEffectView").is_some()
}

#[must_use]
pub fn liquid_glass() -> bool {
    let skipped =
        cfg!(debug_assertions) && std::env::var_os("LINKUNBOUND_NO_LIQUID_GLASS").is_some();
    !skipped && glass_exists()
}

#[must_use]
pub fn reduces_transparency() -> bool {
    NSWorkspace::sharedWorkspace().accessibilityDisplayShouldReduceTransparency()
}

fn material(mtm: MainThreadMarker) -> Retained<NSView> {
    if liquid_glass() {
        return Retained::into_super(NSGlassEffectView::new(mtm));
    }
    let vibrancy = NSVisualEffectView::new(mtm);
    // Popover draws close to solid from macOS 26; menus stay see-through on every version.
    vibrancy.setMaterial(NSVisualEffectMaterial::Menu);
    vibrancy.setBlendingMode(NSVisualEffectBlendingMode::BehindWindow);
    vibrancy.setState(NSVisualEffectState::Active);
    Retained::into_super(vibrancy)
}

fn round(material: &NSView, corner: f64) {
    if glass_exists()
        && let Some(glass) = material.downcast_ref::<NSGlassEffectView>()
    {
        glass.setCornerRadius(corner);
        return;
    }
    material.setWantsLayer(true);
    if let Some(layer) = material.layer() {
        layer.setCornerRadius(corner);
        layer.setMasksToBounds(true);
    }
}

#[allow(unsafe_code)]
fn tone(material: &NSView, dark: bool) {
    // SAFETY: constant names AppKit exports.
    let appearance = NSAppearance::appearanceNamed(unsafe {
        if dark {
            NSAppearanceNameDarkAqua
        } else {
            NSAppearanceNameAqua
        }
    });
    material.setAppearance(appearance.as_deref());
}

fn is_glass(window: &NSWindow) -> bool {
    let name = NSString::from_str(GLASS);
    window
        .contentView()
        .and_then(|view| view.identifier())
        .is_some_and(|id| *id == *name)
}

/// The window holding the material behind the picker, if it wears one.
#[must_use]
pub(crate) fn glass_behind(picker: &NSWindow) -> Option<Retained<NSWindow>> {
    picker.childWindows()?.iter().find(|child| is_glass(child))
}

#[allow(unsafe_code)]
fn glass_window(mtm: MainThreadMarker, picker: &NSWindow) -> Retained<NSWindow> {
    // SAFETY: a fresh allocation, initialised once with constant arguments.
    let window = unsafe {
        NSWindow::initWithContentRect_styleMask_backing_defer(
            mtm.alloc(),
            picker.frame(),
            NSWindowStyleMask::Borderless,
            NSBackingStoreType::Buffered,
            false,
        )
    };
    // SAFETY: the window is owned here and never closed by AppKit on its own.
    unsafe { window.setReleasedWhenClosed(false) };
    window.setOpaque(false);
    window.setBackgroundColor(Some(&NSColor::clearColor()));
    window.setHasShadow(false);
    window.setIgnoresMouseEvents(true);
    window.setHidesOnDeactivate(false);
    window.setCollectionBehavior(picker.collectionBehavior());
    let material = material(mtm);
    material.setIdentifier(Some(&NSString::from_str(GLASS)));
    window.setContentView(Some(&material));
    // SAFETY: both windows live on the main thread; AppKit keeps the child with its parent.
    unsafe { picker.addChildWindow_ordered(&window, NSWindowOrderingMode::Below) };
    keep_with(picker, &window);
    window
}

/// The glass covers what the picker draws, not its frame: AppKit at times gives the frame room
/// above the content the picker never paints.
fn cover(picker: &NSWindow, glass: &NSWindow) {
    if let Some(content) = picker.contentView() {
        glass.setFrame_display(picker.convertRectToScreen(content.frame()), true);
    }
}

#[allow(unsafe_code)]
fn keep_with(picker: &NSWindow, glass: &NSWindow) {
    let follow = {
        let (picker, glass) = (picker.retain(), glass.retain());
        RcBlock::new(move |_: NonNull<NSNotification>| cover(&picker, &glass))
    };
    let centre = NSNotificationCenter::defaultCenter();
    let content = picker.contentView();
    if let Some(content) = &content {
        content.setPostsFrameChangedNotifications(true);
    }
    // SAFETY: AppKit's own names, posted on the main thread where the block runs; the centre keeps
    // the observers for as long as the picker, which lives as long as the app.
    unsafe {
        for name in [NSWindowDidMoveNotification, NSWindowDidResizeNotification] {
            centre.addObserverForName_object_queue_usingBlock(
                Some(name),
                Some(picker),
                None,
                &follow,
            );
        }
        if let Some(content) = &content {
            centre.addObserverForName_object_queue_usingBlock(
                Some(NSViewFrameDidChangeNotification),
                Some(content),
                None,
                &follow,
            );
        }
    }
}

/// The painted sheet is back: the glass behind it would show in its shadow margin.
pub fn take_off_glass(view: isize) {
    let Some(picker) = window_of(view) else {
        return;
    };
    if let Some(glass) = glass_behind(&picker) {
        picker.removeChildWindow(&glass);
        glass.orderOut(None);
    }
}

/// The material lives in a window of its own just behind the picker: AppKit only blends it with
/// what is behind when it is part of a window's content, and the picker's content is winit's.
pub fn wear_glass(view: isize, dark: bool, corner: f64) -> bool {
    let Some(mtm) = MainThreadMarker::new() else {
        return false;
    };
    let Some(picker) = window_of(view) else {
        return false;
    };
    let glass = glass_behind(&picker).unwrap_or_else(|| glass_window(mtm, &picker));
    glass.setLevel(picker.level());
    cover(&picker, &glass);
    let Some(material) = glass.contentView() else {
        return false;
    };
    round(&material, corner);
    tone(&material, dark);
    true
}
