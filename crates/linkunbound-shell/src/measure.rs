//! The person's text size, density and contrast, handed to the picker's Slint globals.

use linkunbound_core::{Density, TextSize};
use slint::ComponentHandle;

use crate::{Measures, Palette, Picker};

/// Row, icon and card in logical pixels; the compact look, all icons, keeps its own.
#[must_use]
pub const fn stands(density: Density) -> (f32, f32, f32) {
    match density {
        Density::Compact => (36.0, 20.0, 44.0),
        Density::Normal => (45.0, 24.0, 52.0),
        Density::Comfortable => (54.0, 28.0, 60.0),
    }
}

pub fn wear(picker: &Picker, size: TextSize, density: Density, contrast: bool) {
    let (row, icon, card) = stands(density);
    let measures = picker.global::<Measures>();
    measures.set_zoom(size.zoom());
    measures.set_row(row);
    measures.set_icon(icon);
    measures.set_card(card);
    picker.global::<Palette>().set_contrast(contrast);
}
