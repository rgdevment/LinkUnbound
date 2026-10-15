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
    rise: 46.0,
    paw: 30.0,
    gap: 70.0,
};
const COMPACT: Pose = Pose {
    head: 66.0,
    rise: 34.0,
    paw: 20.0,
    gap: 46.0,
};

pub struct Mascot {
    head: BearHead,
    paws: BearPaws,
}

impl Mascot {
    pub fn new() -> Result<Self, slint::PlatformError> {
        Ok(Self {
            head: BearHead::new()?,
            paws: BearPaws::new()?,
        })
    }

    /// Only over the glass: on a painted sheet the head would show whole above it, unblurred.
    pub fn follow(&self, picker: &Picker) {
        let pose = match picker.get_look() {
            Look::Bear => &BEAR,
            Look::Compact => &COMPACT,
            _ => {
                self.hide();
                return;
            }
        };
        if !picker.global::<Palette>().get_clear() {
            self.hide();
            return;
        }
        self.head.set_side(pose.head);
        self.paws.set_paw(pose.paw);
        self.paws.set_gap(pose.gap);
        let window = picker.window();
        let scale = window.scale_factor();
        let at = window.position();
        let px = |logical: f32| (logical * scale).round() as i32;
        let centre = if picker.get_look() == Look::Bear {
            at.x + window.size().width as i32 / 2
        } else {
            at.x + px(18.0 + pose.head / 2.0)
        };
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
