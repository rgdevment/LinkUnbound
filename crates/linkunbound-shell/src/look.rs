//! How the picker is drawn: list or tiles, and the material behind it where the system lends one.

use linkunbound_core::PickerStyle;
use linkunbound_shell::{CLASSIC_CORNER, CORNER, HALO, Palette, Picker, place};
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

/// The picker belongs to the click that summoned it, not to the middle of a screen.
pub fn beside_the_pointer(picker: &Picker) {
    let Some((cx, cy)) = crate::host::cursor() else {
        return;
    };
    let Some((x, y, width, height)) = crate::host::work_area_at(cx, cy) else {
        return;
    };
    let scale = if cfg!(target_os = "macos") {
        1.0
    } else {
        f64::from(picker.window().scale_factor())
    };
    let size = |logical: f32| crate::physical(logical, scale);

    let halo = size(picker.get_halo());
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
    // The margin holds the shadow, not the sheet: the sheet's edge is what sits by the pointer,
    // and the transparent rim may hang past the screen without anyone seeing it.
    let (at_x, at_y) = (at_x - halo, at_y - halo);
    #[cfg(target_os = "macos")]
    picker
        .window()
        .set_position(slint::LogicalPosition::new(at_x as f32, at_y as f32));
    #[cfg(not(target_os = "macos"))]
    picker
        .window()
        .set_position(slint::PhysicalPosition::new(at_x, at_y));
    if picker.window().is_visible() {
        follow(picker);
    }
}

/// The mascot comes and goes with the picker it leans on.
pub fn follow(picker: &Picker) {
    if let Some(ui) = crate::ui() {
        ui.mascot.follow(picker);
    }
}

pub fn put_away(picker: &Picker) {
    let _ = picker.hide();
    if let Some(ui) = crate::ui() {
        ui.mascot.hide();
    }
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

#[cfg(test)]
mod tests {
    use super::{acrylic, corner_of, style, wear_backdrop};
    use linkunbound_core::PickerStyle;
    use linkunbound_shell::{CLASSIC_CORNER, CORNER, HALO, Palette, Picker};
    use slint::ComponentHandle;

    fn picker() -> Picker {
        i_slint_backend_testing::init_no_event_loop();
        Picker::new().expect("a window")
    }

    /// The sheet draws its own shadow only where the system lends no material; the list never
    /// draws one.
    #[test]
    fn the_shadow_is_drawn_only_where_nothing_else_draws_one() {
        let picker = picker();
        let clear = acrylic();

        style(&picker, PickerStyle::Sheet);
        assert_eq!(picker.global::<Palette>().get_clear(), clear);
        assert_eq!(picker.get_halo(), if clear { 0.0 } else { HALO });
        assert!((corner_of(&picker) - CORNER).abs() < f64::EPSILON);

        style(&picker, PickerStyle::Classic);
        assert_eq!(picker.get_halo(), 0.0);
        assert!((corner_of(&picker) - CLASSIC_CORNER).abs() < f64::EPSILON);
    }

    /// A window the system will not dress gets the painted sheet back, and the tiles their
    /// shadow, which changes the size the window has to be placed at.
    #[test]
    fn a_refused_material_brings_the_painted_sheet_back() {
        let picker = picker();
        let palette = picker.global::<Palette>();

        style(&picker, PickerStyle::Sheet);
        palette.set_clear(true);
        picker.set_halo(0.0);
        assert!(wear_backdrop(&picker, 0), "the tiles grow by their shadow");
        assert!(!palette.get_clear());
        assert_eq!(picker.get_halo(), HALO);

        style(&picker, PickerStyle::Classic);
        palette.set_clear(true);
        assert!(!wear_backdrop(&picker, 0), "the list has no shadow to add");
        assert!(!palette.get_clear());

        assert!(
            !wear_backdrop(&picker, 0),
            "a painted sheet asks for nothing"
        );
    }
}
