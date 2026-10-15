use objc2::rc::Retained;
use objc2::runtime::AnyClass;
use objc2_app_kit::{
    NSAppearance, NSAppearanceCustomization, NSAppearanceNameAqua, NSAppearanceNameDarkAqua,
    NSAutoresizingMaskOptions, NSGlassEffectView, NSUserInterfaceItemIdentification, NSView,
    NSVisualEffectBlendingMode, NSVisualEffectMaterial, NSVisualEffectState, NSVisualEffectView,
    NSWindowOrderingMode, NSWorkspace,
};
use objc2_foundation::{MainThreadMarker, NSString};

use crate::native::window_of;

const GLASS: &str = "linkunbound-glass";

/// Asked by name: the class only exists from macOS 26, and naming it in code would fail to load
/// on the systems before it.
fn liquid_glass() -> bool {
    let skipped =
        cfg!(debug_assertions) && std::env::var_os("LINKUNBOUND_NO_LIQUID_GLASS").is_some();
    !skipped && AnyClass::get(c"NSGlassEffectView").is_some()
}

#[must_use]
pub fn reduces_transparency() -> bool {
    NSWorkspace::sharedWorkspace().accessibilityDisplayShouldReduceTransparency()
}

fn material(mtm: MainThreadMarker, corner: f64) -> Retained<NSView> {
    if liquid_glass() {
        let glass = NSGlassEffectView::new(mtm);
        glass.setCornerRadius(corner);
        return Retained::into_super(glass);
    }
    let vibrancy = NSVisualEffectView::new(mtm);
    vibrancy.setMaterial(NSVisualEffectMaterial::Popover);
    vibrancy.setBlendingMode(NSVisualEffectBlendingMode::BehindWindow);
    vibrancy.setState(NSVisualEffectState::Active);
    vibrancy.setWantsLayer(true);
    if let Some(layer) = vibrancy.layer() {
        layer.setCornerRadius(corner);
        layer.setMasksToBounds(true);
    }
    Retained::into_super(vibrancy)
}

/// Slint paints into the layer of its own view, so a subview of it would cover the picker: the
/// material goes beside it, underneath, in the view that frames the window.
#[allow(unsafe_code)]
pub fn wear_glass(view: isize, dark: bool, corner: f64) -> bool {
    let Some(mtm) = MainThreadMarker::new() else {
        return false;
    };
    let Some(content) = window_of(view).and_then(|window| window.contentView()) else {
        return false;
    };
    // SAFETY: read on the main thread from a view the window holds.
    let Some(frame) = (unsafe { content.superview() }) else {
        return false;
    };
    let name = NSString::from_str(GLASS);
    for old in frame.subviews().iter() {
        if old.identifier().is_some_and(|id| *id == *name) {
            old.removeFromSuperview();
        }
    }
    let glass = material(mtm, corner);
    glass.setIdentifier(Some(&name));
    glass.setFrame(content.frame());
    glass.setAutoresizingMask(
        NSAutoresizingMaskOptions::ViewWidthSizable | NSAutoresizingMaskOptions::ViewHeightSizable,
    );
    // SAFETY: constant names AppKit exports.
    let appearance = NSAppearance::appearanceNamed(unsafe {
        if dark {
            NSAppearanceNameDarkAqua
        } else {
            NSAppearanceNameAqua
        }
    });
    glass.setAppearance(appearance.as_deref());
    frame.addSubview_positioned_relativeTo(&glass, NSWindowOrderingMode::Below, Some(&content));
    true
}
