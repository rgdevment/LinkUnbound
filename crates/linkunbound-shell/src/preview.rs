//! The picker settings shows so a look can be seen before it is kept: a sample link that no rule
//! answers and no choice opens.

use std::rc::Rc;

use linkunbound_core::PREVIEW;
use linkunbound_shell::Picker;
use slint::ComponentHandle;

use crate::{Arrival, Ui, present};

const SAMPLE: &str = "https://www.linkunbound.app/";

pub fn asked(args: &[String]) -> Option<String> {
    args.iter()
        .any(|a| a == PREVIEW)
        .then(|| PREVIEW.to_owned())
}

/// Whether the line was the preview, shown or not: over a picker in use it waits for nobody, and
/// over a preview it starts again, so a look chosen meanwhile is drawn from scratch. Nothing
/// watches it lose the front: it stays while the person goes back to settings to compare, and
/// goes with Esc or a choice.
pub fn shown_for(ui: &Rc<Ui>, raw: &str) -> bool {
    if raw != PREVIEW {
        return false;
    }
    if ui.picker.window().is_visible() && ui.picker.get_preview() {
        crate::look::put_away(&ui.picker);
        ui.catch_up();
    }
    let sample = Arrival {
        url: SAMPLE.to_owned(),
        source: None,
    };
    if !ui.picker.window().is_visible() && present(&ui.picker, &ui.words.get(), &ui.shown, sample) {
        ui.picker.set_preview(true);
        ui.picker
            .set_source_line(ui.words.get().picker_preview.into());
    }
    true
}

/// A choice in the preview puts it away and opens nothing.
pub fn answered(picker: &Picker) -> bool {
    if !picker.get_preview() {
        return false;
    }
    crate::look::put_away(picker);
    true
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::{SAMPLE, answered, asked, shown_for};
    use crate::Ui;
    use linkunbound_core::{Density, PREVIEW, PickerPosition, PickerStyle, Preferences, TextSize};
    use linkunbound_shell::{Look, Measures, Palette, font};
    use slint::ComponentHandle;

    #[derive(Debug, PartialEq)]
    struct Seen {
        look: Look,
        family: String,
        zoom: f32,
        row: f32,
        contrast: bool,
        centered: bool,
        wanted: (f32, f32),
        size: slint::PhysicalSize,
    }

    /// The look settled the way `catch_up` settles it, then the preview opened and measured once.
    fn first_preview_in(ui: &Rc<Ui>, prefs: &Preferences) -> Seen {
        crate::look::obey(&ui.picker, &ui.notice, prefs);
        assert!(shown_for(ui, PREVIEW));
        let picker = &ui.picker;
        let seen = Seen {
            look: picker.get_look(),
            family: picker.get_family().into(),
            zoom: picker.global::<Measures>().get_zoom(),
            row: picker.global::<Measures>().get_row(),
            contrast: picker.global::<Palette>().get_contrast(),
            centered: picker.get_centered(),
            wanted: (picker.get_wanted_width(), picker.get_wanted_height()),
            size: picker.window().size(),
        };
        crate::look::put_away(picker);
        seen
    }

    fn args(said: &[&str]) -> Vec<String> {
        said.iter().map(|&a| a.to_owned()).collect()
    }

    #[test]
    fn only_the_word_settings_sends_asks_for_the_preview() {
        assert_eq!(
            asked(&args(&["shell.exe", PREVIEW])).as_deref(),
            Some(PREVIEW)
        );
        assert_eq!(asked(&args(&["shell.exe", "https://a.test/"])), None);
    }

    /// The sample opens with nothing behind it: no rule, and a choice only puts it away.
    #[test]
    fn the_preview_shows_the_sample_and_a_choice_opens_nothing() {
        let ui = crate::tests::headless_ui();
        assert!(
            !shown_for(&ui, "https://a.test/"),
            "a link is not the preview"
        );
        assert!(!answered(&ui.picker), "a real picker is answered as usual");

        assert!(shown_for(&ui, PREVIEW));
        assert!(ui.picker.get_preview());
        assert_eq!(ui.shown.borrow().url.as_deref(), Some(SAMPLE));
        assert!(answered(&ui.picker));
    }

    /// Each in a picker of its own, on a thread of its own: Slint starts once per thread.
    fn opened(looks: Vec<Preferences>) -> Vec<Seen> {
        std::thread::spawn(move || {
            let ui = crate::tests::headless_ui();
            looks
                .iter()
                .map(|prefs| first_preview_in(&ui, prefs))
                .collect()
        })
        .join()
        .expect("a picker")
    }

    /// A picker that showed the old look opens the new one already measured and placed, the same
    /// as one that never knew the old: nothing is fitted first and corrected on a second open.
    #[test]
    fn a_changed_look_is_measured_and_placed_on_the_first_open() {
        let before = Preferences {
            shortcut: None,
            ..Preferences::default()
        };
        let other = font::families()
            .into_iter()
            .map(|one| one.name)
            .find(|name| *name != font::system() && font::installed(name));
        let changes = [
            (
                "text size",
                Preferences {
                    text_size: TextSize::Large,
                    ..before.clone()
                },
            ),
            (
                "density",
                Preferences {
                    density: Density::Comfortable,
                    ..before.clone()
                },
            ),
            (
                "contrast",
                Preferences {
                    high_contrast: true,
                    ..before.clone()
                },
            ),
            (
                "style",
                Preferences {
                    picker_style: PickerStyle::Classic,
                    ..before.clone()
                },
            ),
            (
                "position",
                Preferences {
                    picker_position: PickerPosition::Center,
                    ..before.clone()
                },
            ),
            (
                "font",
                Preferences {
                    font: other.clone(),
                    ..before.clone()
                },
            ),
        ];
        let old = opened(vec![before.clone()]).remove(0);
        // Linux CI has one family, so there is no other font to change to there.
        for (what, after) in changes
            .into_iter()
            .filter(|(what, _)| *what != "font" || other.is_some())
        {
            let seen = opened(vec![before.clone(), after.clone()]);
            let fresh = opened(vec![after]).remove(0);
            assert_eq!(seen[0], old, "{what}");
            assert_eq!(seen[1], fresh, "{what}");
            assert_ne!(seen[1], old, "{what} changed nothing the picker shows");
        }
        if cfg!(any(windows, target_os = "macos")) {
            assert!(other.is_some(), "a second installed family");
        }
    }

    /// Read under a picker on screen, a new look would redraw it where the old one fit.
    #[test]
    fn settings_are_not_read_under_a_picker_on_screen() {
        let ui = crate::tests::headless_ui();
        assert!(shown_for(&ui, PREVIEW));
        let long_ago = Some(std::time::SystemTime::UNIX_EPOCH);
        ui.prefs_seen.set(long_ago);
        assert!(!ui.settings_moved());
        assert_eq!(
            ui.prefs_seen.get(),
            long_ago,
            "left for when it is put away"
        );
        crate::look::put_away(&ui.picker);
        assert!(ui.settings_moved());
    }
}
