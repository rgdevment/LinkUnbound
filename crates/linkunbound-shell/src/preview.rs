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
    use super::{SAMPLE, answered, asked, shown_for};
    use linkunbound_core::PREVIEW;

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
}
