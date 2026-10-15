//! The bear that peeks over the picker: its head under the glass, its paws on the edge.

use linkunbound_shell::{BearHead, BearPaws, Palette, Picker};
use slint::ComponentHandle;

use crate::native_handle;

const HEAD: f32 = 92.0;
const HEAD_RISE: f32 = 46.0;
const PAW: f32 = 30.0;
const PAW_GAP: f32 = 70.0;

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

    pub fn follow(&self, picker: &Picker) {
        if !picker.global::<Palette>().get_clear() {
            self.hide();
            return;
        }
        let window = picker.window();
        let scale = window.scale_factor();
        let at = window.position();
        let mid = at.x + window.size().width as i32 / 2;
        let px = |logical: f32| (logical * scale).round() as i32;
        self.head
            .window()
            .set_position(slint::PhysicalPosition::new(
                mid - px(HEAD / 2.0),
                at.y - px(HEAD_RISE),
            ));
        self.paws
            .window()
            .set_position(slint::PhysicalPosition::new(
                mid - px(PAW_GAP / 2.0 + PAW),
                at.y - px(PAW * 0.3),
            ));
        let _ = self.head.show();
        let _ = self.paws.show();
        stack(&self.head, &self.paws, picker);
    }

    pub fn hide(&self) {
        let _ = self.head.hide();
        let _ = self.paws.hide();
    }
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
