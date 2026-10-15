//! How the picker is drawn: list or tiles, and the material behind it where the system lends one.

use linkunbound_core::PickerStyle;
use linkunbound_shell::{CLASSIC_CORNER, CORNER, HALO, Palette, Picker};
use slint::ComponentHandle;

pub fn style(picker: &Picker, chosen: PickerStyle) {
    let classic = chosen != PickerStyle::Sheet;
    let clear = acrylic();
    picker.set_classic(classic);
    picker.global::<Palette>().set_clear(clear);
    picker.set_halo(if classic || clear { 0.0 } else { HALO });
}

pub fn corner_of(picker: &Picker) -> f64 {
    if picker.get_classic() {
        CLASSIC_CORNER
    } else {
        CORNER
    }
}

/// Refused, the material would leave the clear sheet showing the desktop through it, so the
/// painted sheet and its shadow come back. Returns whether the window changed size.
pub fn wear_backdrop(picker: &Picker, handle: isize) -> bool {
    let palette = picker.global::<Palette>();
    if !palette.get_clear() || wear_acrylic(handle, !palette.get_light()) {
        return false;
    }
    palette.set_clear(false);
    if picker.get_classic() {
        return false;
    }
    picker.set_halo(HALO);
    true
}

#[cfg(windows)]
fn acrylic() -> bool {
    linkunbound_win::acrylic_is_available()
}

#[cfg(windows)]
fn wear_acrylic(window: isize, dark: bool) -> bool {
    linkunbound_win::wear_acrylic(window, dark)
}

#[cfg(not(windows))]
fn acrylic() -> bool {
    false
}

#[cfg(not(windows))]
fn wear_acrylic(_window: isize, _dark: bool) -> bool {
    false
}
