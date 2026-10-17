//! The bear that peeks over the picker: its head under the glass, its paws on the edge.

use linkunbound_shell::{BearHead, BearPaws, Look, Palette, Picker};
use slint::ComponentHandle;

/// Sizes in logical pixels: the head, how far it rises over the edge, a paw, the room between
/// the paws, and where across the picker the head is centred.
struct Pose {
    head: f32,
    rise: f32,
    paw: f32,
    gap: f32,
}

const BEAR: Pose = Pose {
    head: 92.0,
    rise: 52.0,
    paw: 30.0,
    gap: 70.0,
};
const COMPACT: Pose = Pose {
    head: 66.0,
    rise: 34.0,
    paw: 20.0,
    gap: 46.0,
};

/// The pose the look calls for, if the bear shows at all. Its head sits under the picker, so on
/// glass the hidden half shows blurred and on a painted sheet it is simply behind it.
fn pose(picker: &Picker) -> Option<&'static Pose> {
    if picker.global::<Palette>().get_solid() {
        return None;
    }
    match picker.get_look() {
        Look::Bear => Some(&BEAR),
        Look::Compact => Some(&COMPACT),
        _ => None,
    }
}

/// How far above the picker the bear reaches, in logical pixels.
pub fn rise(picker: &Picker) -> f32 {
    pose(picker).map_or(0.0, |pose| pose.rise)
}

pub struct Mascot {
    head: BearHead,
    paws: BearPaws,
    /// Where the picker was last put, as asked: Slint moves the window later, so reading it back
    /// right after gives the place it is leaving.
    anchor: std::cell::Cell<Option<(i32, i32)>>,
}

impl Mascot {
    pub fn new() -> Result<Self, slint::PlatformError> {
        Ok(Self {
            head: BearHead::new()?,
            paws: BearPaws::new()?,
            anchor: std::cell::Cell::new(None),
        })
    }

    pub fn anchor(&self, x: i32, y: i32) {
        self.anchor.set(Some((x, y)));
    }

    pub fn follow(&self, picker: &Picker) {
        let Some(pose) = pose(picker) else {
            self.hide();
            return;
        };
        self.head.set_side(pose.head);
        self.paws.set_paw(pose.paw);
        self.paws.set_gap(pose.gap);
        // Measured from what the look asks for, not from the window: right after a change of
        // look or place the window still reports the size and spot it had before.
        let window = picker.window();
        let scale = placing_scale(window);
        let (x, y) = self.anchor.get().unwrap_or_else(|| where_it_is(window));
        let (head, paws) = spots(
            pose,
            (x, y),
            picker.get_wanted_width(),
            picker.get_halo(),
            scale,
        );
        put(
            self.head.window(),
            head,
            (self.head.get_wide(), self.head.get_tall()),
        );
        put(
            self.paws.window(),
            paws,
            (self.paws.get_wide(), self.paws.get_tall()),
        );
        self.mood(0.0, picker.get_private_on());
        let _ = self.head.show();
        let _ = self.paws.show();
        stack(&self.head, &self.paws, picker);
    }

    pub fn mood(&self, aim: f32, masked: bool) {
        self.head.set_look(aim);
        self.head.set_masked(masked);
    }

    pub fn hide(&self) {
        let _ = self.head.hide();
        let _ = self.paws.hide();
    }
}

/// Where the head and the paws go for a picker put at `at`, in the units it was put in.
fn spots(
    pose: &Pose,
    (x, y): (i32, i32),
    width: f32,
    halo: f32,
    scale: f32,
) -> ((i32, i32), (i32, i32)) {
    let px = |logical: f32| (logical * scale).round() as i32;
    let centre = x + px(width / 2.0);
    // A painted sheet keeps a margin for its shadow; the bear leans on the sheet, not on it.
    let top = y + px(halo);
    (
        (centre - px(pose.head / 2.0), top - px(pose.rise)),
        (
            centre - px(pose.gap / 2.0 + pose.paw),
            top - px(pose.paw * 0.3),
        ),
    )
}

/// The picker is placed in points on macOS and in pixels elsewhere; the bear goes in the same units.
fn placing_scale(window: &slint::Window) -> f32 {
    if cfg!(target_os = "macos") {
        1.0
    } else {
        window.scale_factor()
    }
}

fn where_it_is(window: &slint::Window) -> (i32, i32) {
    let at = window.position();
    if cfg!(target_os = "macos") {
        let at = at.to_logical(window.scale_factor());
        (at.x.round() as i32, at.y.round() as i32)
    } else {
        (at.x, at.y)
    }
}

/// On macOS the size goes first: AppKit keeps a window's bottom edge when Slint resizes it later,
/// which would sink the head of a smaller pose into the picker.
fn put(window: &slint::Window, (x, y): (i32, i32), (wide, tall): (f32, f32)) {
    #[cfg(target_os = "macos")]
    {
        window.set_size(slint::LogicalSize::new(wide, tall));
        window.set_position(slint::LogicalPosition::new(x as f32, y as f32));
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (wide, tall);
        window.set_position(slint::PhysicalPosition::new(x, y));
    }
}

/// The picker says where the bear should look and whether it wears the mask, and an empty slot
/// sends the person to settings to fill it.
pub fn listen(picker: &Picker) {
    picker.on_mood(|aim, masked| {
        if let Some(ui) = crate::ui() {
            ui.mascot.mood(aim, masked);
        }
    });
    let handle = picker.as_weak();
    picker.on_add_browser(move || {
        if let Some(picker) = handle.upgrade() {
            crate::look::put_away(&picker);
        }
        crate::run_settings(&[linkunbound_core::BROWSERS]);
    });
}

#[cfg(windows)]
fn stack(head: &BearHead, paws: &BearPaws, picker: &Picker) {
    use crate::native_handle;
    let (Some(head), Some(paws), Some(picker)) = (
        native_handle(head.window()),
        native_handle(paws.window()),
        native_handle(picker.window()),
    ) else {
        return;
    };
    for window in [head, paws] {
        linkunbound_win::keep_off_the_taskbar(window);
        linkunbound_win::never_activates(window);
    }
    linkunbound_win::tuck_under(head, picker);
}

#[cfg(target_os = "macos")]
fn stack(head: &BearHead, paws: &BearPaws, picker: &Picker) {
    use crate::native_handle;
    let (Some(head), Some(paws), Some(picker)) = (
        native_handle(head.window()),
        native_handle(paws.window()),
        native_handle(picker.window()),
    ) else {
        return;
    };
    linkunbound_mac::tuck_beside(head, picker, true);
    linkunbound_mac::tuck_beside(paws, picker, false);
}

#[cfg(not(any(windows, target_os = "macos")))]
fn stack(_head: &BearHead, _paws: &BearPaws, _picker: &Picker) {}

#[cfg(test)]
mod tests {
    use super::{BEAR, COMPACT, Mascot, rise, spots};
    use linkunbound_shell::{Look, Palette, Picker};
    use slint::ComponentHandle;

    fn picker(look: Look, clear: bool) -> Picker {
        let picker = Picker::new().expect("a window");
        picker.set_look(look);
        picker.global::<Palette>().set_clear(clear);
        picker
    }

    /// The bear rises over the glass only, and only in the two looks drawn for it.
    #[test]
    fn the_bear_needs_room_only_where_it_shows() {
        i_slint_backend_testing::init_no_event_loop();
        assert_eq!(rise(&picker(Look::Bear, true)), BEAR.rise);
        assert_eq!(rise(&picker(Look::Compact, true)), COMPACT.rise);
        assert_eq!(rise(&picker(Look::List, true)), 0.0);
        assert_eq!(
            rise(&picker(Look::Bear, false)),
            BEAR.rise,
            "a painted sheet has the bear behind it too"
        );
    }

    #[test]
    fn the_bear_comes_with_the_picker_and_follows_its_mood() {
        i_slint_backend_testing::init_no_event_loop();
        let mascot = Mascot::new().expect("a bear");
        let over = picker(Look::Bear, true);
        over.set_private_on(true);
        mascot.follow(&over);
        assert!(mascot.head.window().is_visible());
        assert!(mascot.paws.window().is_visible());
        assert!(mascot.head.get_masked());
        assert_eq!(mascot.head.get_side(), BEAR.head);

        mascot.mood(0.7, false);
        assert_eq!(mascot.head.get_look(), 0.7);
        assert!(!mascot.head.get_masked());

        mascot.follow(&picker(Look::Tiles, true));
        assert!(!mascot.head.window().is_visible());
        assert!(!mascot.paws.window().is_visible());
    }

    /// The head stands over the middle of the sheet and rises its pose's height over it, the paws
    /// on the edge, whatever the scale the picker was placed at.
    #[test]
    fn the_bear_leans_on_the_middle_of_the_sheet() {
        let (head, paws) = spots(&BEAR, (100, 300), 400.0, 0.0, 1.0);
        assert_eq!(head, (300 - 46, 300 - 52));
        assert_eq!(paws, (300 - 65, 300 - 9));

        let (head, _) = spots(&BEAR, (200, 600), 400.0, 18.0, 2.0);
        assert_eq!(
            head,
            (600 - 92, 636 - 104),
            "pixels on a dense screen, under a shadow margin"
        );
    }
}
