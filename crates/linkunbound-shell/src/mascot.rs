//! The bear that peeks over the picker: its head under the glass, its paws on the edge.

use linkunbound_shell::{BearHead, BearPaws, Look, Palette, Picker};
use slint::ComponentHandle;

use crate::native_handle;

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
    rise: 62.0,
    paw: 30.0,
    gap: 70.0,
};
const COMPACT: Pose = Pose {
    head: 66.0,
    rise: 34.0,
    paw: 20.0,
    gap: 46.0,
};

/// The pose the look calls for, if the bear shows at all: only over the glass, since on a painted
/// sheet the head would show whole above it, unblurred.
fn pose(picker: &Picker) -> Option<&'static Pose> {
    if !picker.global::<Palette>().get_clear() {
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
        let scale = window.scale_factor();
        let (x, y) = self
            .anchor
            .get()
            .unwrap_or((window.position().x, window.position().y));
        let at = slint::PhysicalPosition::new(x, y);
        let px = |logical: f32| (logical * scale).round() as i32;
        let centre = at.x + px(picker.get_wanted_width() / 2.0);
        self.head
            .window()
            .set_position(slint::PhysicalPosition::new(
                centre - px(pose.head / 2.0),
                at.y - px(pose.rise),
            ));
        self.paws
            .window()
            .set_position(slint::PhysicalPosition::new(
                centre - px(pose.gap / 2.0 + pose.paw),
                at.y - px(pose.paw * 0.3),
            ));
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

/// The picker says where the bear should look and whether it wears the mask.
pub fn listen(picker: &Picker) {
    picker.on_mood(|aim, masked| {
        if let Some(ui) = crate::ui() {
            ui.mascot.mood(aim, masked);
        }
    });
}

#[cfg(windows)]
fn stack(head: &BearHead, paws: &BearPaws, picker: &Picker) {
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

#[cfg(not(windows))]
fn stack(_head: &BearHead, _paws: &BearPaws, _picker: &Picker) {}

#[cfg(test)]
mod tests {
    use super::{BEAR, COMPACT, Mascot, rise};
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
        assert_eq!(rise(&picker(Look::Bear, false)), 0.0, "no glass, no bear");
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
}
