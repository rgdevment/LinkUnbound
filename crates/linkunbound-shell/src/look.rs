//! How the picker is drawn: which look, and the material behind it where the system lends one.

use linkunbound_core::PickerStyle;
use linkunbound_shell::{CLASSIC_CORNER, CORNER, HALO, Look, Palette, Picker, place};
use slint::ComponentHandle;

pub fn style(picker: &Picker, chosen: PickerStyle) {
    let look = match chosen {
        PickerStyle::Classic => Look::List,
        PickerStyle::Sheet => Look::Tiles,
        PickerStyle::Compact => Look::Compact,
        PickerStyle::Bear => Look::Bear,
    };
    let clear = native_glass();
    picker.set_look(look);
    let palette = picker.global::<Palette>();
    palette.set_clear(clear);
    palette.set_tint(tint());
    palette.set_glass_corner(if cfg!(target_os = "macos") {
        corner_of(picker) as f32
    } else {
        DWM_CORNER
    });
    palette.set_see_through(see_through());
    picker.set_halo(if look == Look::List || clear {
        0.0
    } else {
        HALO
    });
}

/// How much of the picker's tone is laid over the material: Acrylic comes milky, Liquid Glass
/// and menu vibrancy let far more of the desktop through.
#[cfg(target_os = "macos")]
fn tint() -> f32 {
    if linkunbound_mac::liquid_glass() {
        0.55
    } else {
        0.5
    }
}

#[cfg(not(target_os = "macos"))]
fn tint() -> f32 {
    0.77
}

const DWM_CORNER: f32 = 8.0;

pub fn corner_of(picker: &Picker) -> f64 {
    if picker.get_look() == Look::List {
        CLASSIC_CORNER
    } else {
        CORNER
    }
}

/// Refused, the material would leave the clear sheet showing the desktop through it, so the
/// painted sheet and its shadow come back. Returns whether the window changed size.
pub fn wear_backdrop(picker: &Picker, handle: isize) -> bool {
    let palette = picker.global::<Palette>();
    if !palette.get_clear() {
        take_off_glass(handle);
        return false;
    }
    if wear_glass(handle, !palette.get_light(), corner_of(picker)) {
        return false;
    }
    palette.set_clear(false);
    if picker.get_look() == Look::List {
        return false;
    }
    picker.set_halo(HALO);
    true
}

/// The picker belongs to the click that summoned it, not to the middle of a screen.
pub fn beside_the_pointer(picker: &Picker) {
    let Some(pointer) = crate::host::cursor() else {
        return;
    };
    let Some(area) = crate::host::work_area_at(pointer.0, pointer.1) else {
        return;
    };
    place_near(picker, pointer, area);
}

/// Where the picker goes for a pointer and the screen area around it, in the units the window is
/// placed in. Returns the corner it was put at.
fn place_near(
    picker: &Picker,
    (cx, cy): (i32, i32),
    (x, y, width, height): (i32, i32, i32, i32),
) -> (i32, i32) {
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
        // The bear rises over the sheet, so the screen it may use starts that much lower.
        place::Bounds {
            x,
            y: y + size(crate::mascot::rise(picker)),
            width,
            height: height - size(crate::mascot::rise(picker)),
        },
    );
    // The margin holds the shadow, not the sheet: the sheet's edge is what sits by the pointer,
    // and the transparent rim may hang past the screen without anyone seeing it.
    let (at_x, at_y) = (at_x - halo, at_y - halo);
    if let Some(ui) = crate::ui() {
        ui.mascot.anchor(at_x, at_y);
    }
    // AppKit keeps a window's bottom edge when it grows, so the size Slint would set later would
    // lift the picker off the place asked for, and off the bear.
    #[cfg(target_os = "macos")]
    {
        picker.window().set_size(slint::LogicalSize::new(
            picker.get_wanted_width(),
            picker.get_wanted_height(),
        ));
        picker
            .window()
            .set_position(slint::LogicalPosition::new(at_x as f32, at_y as f32));
    }
    #[cfg(not(target_os = "macos"))]
    picker
        .window()
        .set_position(slint::PhysicalPosition::new(at_x, at_y));
    if picker.window().is_visible() {
        follow(picker);
    }
    (at_x, at_y)
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
fn see_through() -> bool {
    linkunbound_win::transparency_is_on()
}

#[cfg(target_os = "macos")]
fn see_through() -> bool {
    !linkunbound_mac::reduces_transparency()
}

#[cfg(not(any(windows, target_os = "macos")))]
fn see_through() -> bool {
    true
}

#[cfg(windows)]
fn native_glass() -> bool {
    // A debug build can be told to paint, to see on Windows 11 what Windows 10 shows.
    let painted = cfg!(debug_assertions) && std::env::var_os("LINKUNBOUND_NO_ACRYLIC").is_some();
    linkunbound_win::acrylic_is_available() && !painted
}

#[cfg(windows)]
fn wear_glass(window: isize, dark: bool, _corner: f64) -> bool {
    linkunbound_win::wear_acrylic(window, dark)
}

/// Vibrancy is there on every macOS the app runs on, Liquid Glass from 26; only the person asking
/// for less transparency sends the picker back to its painted sheet.
#[cfg(target_os = "macos")]
fn native_glass() -> bool {
    !linkunbound_mac::reduces_transparency()
}

#[cfg(target_os = "macos")]
fn wear_glass(view: isize, dark: bool, corner: f64) -> bool {
    linkunbound_mac::wear_glass(view, dark, corner)
}

#[cfg(target_os = "macos")]
fn take_off_glass(view: isize) {
    linkunbound_mac::take_off_glass(view);
}

/// Acrylic is a window attribute DWM drops when the system turns transparency off.
#[cfg(not(target_os = "macos"))]
fn take_off_glass(_window: isize) {}

#[cfg(not(any(windows, target_os = "macos")))]
fn native_glass() -> bool {
    false
}

#[cfg(not(any(windows, target_os = "macos")))]
fn wear_glass(_window: isize, _dark: bool, _corner: f64) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::{corner_of, native_glass, place_near, style, wear_backdrop};
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
        let clear = native_glass();

        style(&picker, PickerStyle::Sheet);
        assert_eq!(picker.global::<Palette>().get_clear(), clear);
        assert_eq!(picker.get_halo(), if clear { 0.0 } else { HALO });
        assert!((corner_of(&picker) - CORNER).abs() < f64::EPSILON);

        style(&picker, PickerStyle::Classic);
        assert_eq!(picker.get_halo(), 0.0);
        assert!((corner_of(&picker) - CLASSIC_CORNER).abs() < f64::EPSILON);
    }

    /// The bear rises over the sheet, so a picker near the top of the screen is put that much
    /// lower; a look without the bear goes right to the edge.
    #[test]
    fn a_picker_by_the_top_edge_keeps_room_for_the_bear() {
        let picker = picker();
        let screen = (0, 0, 1920, 1080);
        let near_the_top = (900, 2);

        style(&picker, PickerStyle::Classic);
        let (_, list_y) = place_near(&picker, near_the_top, screen);

        style(&picker, PickerStyle::Bear);
        picker.set_halo(0.0);
        let (_, bear_y) = place_near(&picker, near_the_top, screen);
        let rise = crate::mascot::rise(&picker);
        assert!(
            f64::from(bear_y) >= f64::from(rise) * f64::from(picker.window().scale_factor()) - 1.0,
            "the bear's head stays on the screen: {bear_y} for a rise of {rise}"
        );
        assert!(bear_y > list_y, "the list needs no room above it");
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
