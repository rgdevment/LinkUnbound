use linkunbound_core::Strings;
use linkunbound_shell::Notice;
use slint::ComponentHandle;

use crate::{host, native_handle, physical, store, ui};

const NOTICE_SECONDS: i32 = 6;
const NOTICE_MARGIN: f32 = 16.0;

/// Never focused: it must not take the keyboard from whatever is being done.
pub fn flash(notice: &Notice, headline: String, reason: String, undo: Option<&str>) {
    notice.set_headline(headline.into());
    notice.set_reason(reason.into());
    notice.set_undo_label(undo.unwrap_or_default().into());
    notice.set_undoable(undo.is_some());
    notice.set_left(NOTICE_SECONDS);
    let _ = notice.show();
    if let Some(handle) = native_handle(notice.window()) {
        host::keep_off_the_taskbar(handle, linkunbound_shell::CLASSIC_CORNER);
        host::never_activates(handle);
    }
    in_the_corner(notice);
    #[cfg(target_os = "macos")]
    if !ui().is_some_and(|ui| ui.picker.window().is_visible()) {
        host::let_whoever_opens_next_come_forward();
    }
}

pub fn noted(area: &'static str, what: &str) {
    linkunbound_core::note(store().dir(), area, what);
}

pub fn troubled(area: &'static str, what: &str, said: impl FnOnce(&Strings) -> (String, String)) {
    noted(area, what);
    let Some(ui) = ui() else { return };
    let (headline, reason) = said(&ui.words.get());
    ui.firing.replace(None);
    flash(&ui.notice, headline, reason, None);
    ui.count_down();
}

/// The corner of the screen the click happened on, where a notice is looked for; left to the
/// window manager it opened wherever the last one did, usually another screen, and the six
/// seconds passed unseen.
fn in_the_corner(notice: &Notice) {
    let Some((cx, cy)) = host::cursor() else {
        return;
    };
    let Some((x, y, width, height)) = host::work_area_at(cx, cy) else {
        return;
    };
    let scale = if cfg!(target_os = "macos") {
        1.0
    } else {
        f64::from(notice.window().scale_factor())
    };
    let size = |logical: f32| physical(logical, scale);
    let at_x = x + width - size(notice.get_wanted_width()) - size(NOTICE_MARGIN);
    let at_y = y + height - size(notice.get_wanted_height()) - size(NOTICE_MARGIN);
    #[cfg(target_os = "macos")]
    notice
        .window()
        .set_position(slint::LogicalPosition::new(at_x as f32, at_y as f32));
    #[cfg(not(target_os = "macos"))]
    notice
        .window()
        .set_position(slint::PhysicalPosition::new(at_x, at_y));
}

#[cfg(test)]
mod tests {
    use super::flash;
    use linkunbound_shell::Notice;

    #[test]
    fn a_notice_about_trouble_offers_nothing_to_undo() {
        i_slint_backend_testing::init_no_event_loop();
        let notice = Notice::new().expect("a notice");

        flash(&notice, "Opened".into(), "by a rule".into(), Some("Undo"));
        assert!(notice.get_undoable());
        assert_eq!(notice.get_undo_label(), "Undo");

        flash(
            &notice,
            "The rule could not open Firefox".into(),
            "why".into(),
            None,
        );
        assert!(!notice.get_undoable());
        assert_eq!(notice.get_headline(), "The rule could not open Firefox");
    }
}
