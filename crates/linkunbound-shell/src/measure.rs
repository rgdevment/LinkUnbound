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

#[cfg(test)]
mod tests {
    use super::{stands, wear};
    use crate::{Listed, Look, Measures, Palette, Picker, dress};
    use linkunbound_core::{Density, Language, TextSize};
    use slint::ComponentHandle;

    fn picker() -> Picker {
        thread_local!(static BACKEND: () = i_slint_backend_testing::init_no_event_loop());
        BACKEND.with(|()| ());
        Picker::new().expect("a window")
    }

    fn browsers(count: usize) -> Vec<Listed> {
        (0..count)
            .map(|at| Listed {
                browser_id: format!("browser{at}"),
                profile_id: None,
                name: format!("Browser {at}"),
                profile: String::new(),
                icon: None,
                can_private: true,
                key: None,
            })
            .collect()
    }

    fn opened(look: Look, count: usize) -> Picker {
        let picker = picker();
        picker.set_look(look);
        dress(
            &picker,
            &Language::Spanish.strings(),
            "https://docs.google.com/a",
            None,
            &browsers(count),
        );
        picker
    }

    fn worn(look: Look, count: usize, size: TextSize, density: Density) -> Picker {
        let picker = opened(look, count);
        wear(&picker, size, density, false);
        picker
    }

    #[test]
    fn each_density_has_its_row_icon_and_card() {
        assert_eq!(stands(Density::Compact), (36.0, 20.0, 44.0));
        assert_eq!(stands(Density::Normal), (45.0, 24.0, 52.0));
        assert_eq!(stands(Density::Comfortable), (54.0, 28.0, 60.0));
    }

    #[test]
    fn wearing_a_choice_sets_the_measures_and_the_contrast() {
        let picker = picker();
        wear(&picker, TextSize::Large, Density::Comfortable, true);

        let measures = picker.global::<Measures>();
        assert_eq!(measures.get_zoom(), 1.13);
        assert_eq!(measures.get_row(), 54.0);
        assert_eq!(measures.get_icon(), 28.0);
        assert_eq!(measures.get_card(), 60.0);
        assert!(picker.global::<Palette>().get_contrast());

        wear(&picker, TextSize::Small, Density::Compact, false);
        assert_eq!(measures.get_zoom(), 0.87);
        assert_eq!(measures.get_row(), 36.0);
        assert!(!picker.global::<Palette>().get_contrast());
    }

    #[test]
    fn untouched_preferences_draw_the_list_as_it_always_was() {
        for rows in [1, 2, 5, 9, 12] {
            let picker = opened(Look::List, rows);
            let shown = rows.min(9) as f32;
            let today = 60.0 + 1.0 + (shown * 45.0 + 11.0) + 1.0 + 74.0;
            assert_eq!(picker.get_wanted_height(), today, "{rows} rows");
            assert_eq!(picker.get_drawn_icon_side(), 24.0);
        }
        let width = opened(Look::List, 3).get_wanted_width();
        assert!((380.0..=560.0).contains(&width), "{width}");
    }

    #[test]
    fn untouched_preferences_draw_the_cards_as_they_always_were() {
        for (rows, lines) in [(1, 1.0), (3, 1.0), (6, 2.0), (12, 4.0), (15, 4.0)] {
            for look in [Look::Tiles, Look::Bear] {
                let picker = opened(look, rows);
                let today = 80.0 + 1.0 + (lines * 58.0 + 10.0) + 1.0 + 74.0;
                assert_eq!(picker.get_wanted_height(), today, "{rows} rows");
                assert_eq!(picker.get_drawn_icon_side(), 32.0);
            }
        }
    }

    #[test]
    fn the_density_moves_the_list_by_nine_rows_at_most() {
        let height =
            |density, rows| worn(Look::List, rows, TextSize::Normal, density).get_wanted_height();

        for rows in [9, 12] {
            let normal = height(Density::Normal, rows);
            assert_eq!(height(Density::Comfortable, rows) - normal, 9.0 * 9.0);
            assert_eq!(normal - height(Density::Compact, rows), 9.0 * 9.0);
        }
        assert_eq!(
            height(Density::Comfortable, 12),
            height(Density::Comfortable, 9),
            "twelve browsers still show nine rows"
        );
        assert_eq!(
            height(Density::Comfortable, 3) - height(Density::Normal, 3),
            3.0 * 9.0
        );
    }

    #[test]
    fn the_icons_of_the_list_follow_the_density() {
        for (density, side) in [
            (Density::Compact, 20.0),
            (Density::Normal, 24.0),
            (Density::Comfortable, 28.0),
        ] {
            let list = worn(Look::List, 3, TextSize::Normal, density);
            assert_eq!(list.get_drawn_icon_side(), side);
            let cards = worn(Look::Tiles, 3, TextSize::Normal, density);
            assert_eq!(cards.get_drawn_icon_side(), 32.0, "cards keep their icon");
        }
    }

    #[test]
    fn the_cards_follow_the_density_and_stop_at_four_lines() {
        let height =
            |density, rows| worn(Look::Tiles, rows, TextSize::Normal, density).get_wanted_height();

        assert_eq!(
            height(Density::Comfortable, 3) - height(Density::Normal, 3),
            8.0
        );
        assert_eq!(
            height(Density::Normal, 3) - height(Density::Compact, 3),
            8.0
        );
        assert_eq!(
            height(Density::Comfortable, 12) - height(Density::Normal, 12),
            4.0 * 8.0
        );
        assert_eq!(
            height(Density::Comfortable, 15),
            height(Density::Comfortable, 12)
        );
    }

    #[test]
    fn the_compact_look_ignores_the_density() {
        let size = |density| {
            let picker = worn(Look::Compact, 4, TextSize::Normal, density);
            (picker.get_wanted_width(), picker.get_wanted_height())
        };
        assert_eq!(size(Density::Compact), size(Density::Normal));
        assert_eq!(size(Density::Comfortable), size(Density::Normal));
    }

    #[test]
    fn larger_letters_make_the_head_and_foot_taller_and_never_narrower() {
        for look in [Look::List, Look::Tiles] {
            let at = |size| worn(look, 3, size, Density::Normal);
            let (small, normal, large) = (
                at(TextSize::Small),
                at(TextSize::Normal),
                at(TextSize::Large),
            );

            assert!(large.get_wanted_height() > normal.get_wanted_height());
            assert!(large.get_wanted_width() >= normal.get_wanted_width());
            assert_eq!(
                small.get_wanted_height(),
                normal.get_wanted_height(),
                "smaller letters keep the head and the foot they had"
            );
            assert!(small.get_wanted_width() >= 380.0);
        }
    }

    #[test]
    fn a_short_screen_shrinks_the_list_but_never_below_one_row() {
        let picker = worn(Look::List, 12, TextSize::Normal, Density::Comfortable);
        let roomy = picker.get_wanted_height();

        picker.set_room_height(400.0);
        let squeezed = picker.get_wanted_height();
        assert!(squeezed <= 400.0, "{squeezed}");
        assert!(squeezed < roomy);

        picker.set_room_height(1.0);
        assert_eq!(
            picker.get_wanted_height(),
            60.0 + 1.0 + (54.0 + 11.0) + 1.0 + 74.0,
            "one row always stays"
        );

        picker.set_room_height(100_000.0);
        assert_eq!(picker.get_wanted_height(), roomy);
    }

    #[test]
    fn a_short_screen_shrinks_the_cards_but_never_below_one_line() {
        let picker = worn(Look::Tiles, 12, TextSize::Normal, Density::Normal);
        let roomy = picker.get_wanted_height();

        picker.set_room_height(350.0);
        let squeezed = picker.get_wanted_height();
        assert!(squeezed <= 350.0, "{squeezed}");
        assert!(squeezed < roomy);

        picker.set_room_height(1.0);
        assert_eq!(
            picker.get_wanted_height(),
            80.0 + 1.0 + (52.0 + 16.0) + 1.0 + 74.0
        );
    }

    #[test]
    fn a_narrow_screen_caps_the_sheet_width() {
        let picker = worn(Look::List, 3, TextSize::Normal, Density::Normal);
        picker.set_room_width(300.0);
        assert_eq!(picker.get_wanted_width(), 300.0);
    }

    #[test]
    fn a_roomy_screen_changes_nothing() {
        let plain = worn(Look::List, 12, TextSize::Normal, Density::Normal);
        let roomy = worn(Look::List, 12, TextSize::Normal, Density::Normal);
        roomy.set_room_height(1080.0 - 16.0);
        roomy.set_room_width(1920.0 - 16.0);
        assert_eq!(roomy.get_wanted_height(), plain.get_wanted_height());
        assert_eq!(roomy.get_wanted_width(), plain.get_wanted_width());
    }

    #[test]
    fn the_update_strip_grows_with_the_letters() {
        use crate::{Strip, show_strip};
        let strip = Strip {
            stage: "offer",
            line: "Versión 2.0.1 lista".to_owned(),
            detail: String::new(),
            button: "Actualizar".to_owned(),
            far: 0,
        };
        for (size, zoom) in [(TextSize::Normal, 1.0_f32), (TextSize::Large, 1.13)] {
            let picker = worn(Look::List, 3, size, Density::Normal);
            let plain = picker.get_wanted_height();
            show_strip(&picker, Some(&strip));
            let grown = picker.get_wanted_height() - plain;
            let wanted = 46.0 * (1.0_f32).max(zoom * 0.96);
            assert!((grown - wanted).abs() < 0.01, "{size:?}: {grown}");
        }
    }
}
