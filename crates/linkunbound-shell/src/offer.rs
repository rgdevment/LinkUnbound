use std::rc::Rc;
use std::time::Duration;

use linkunbound_core::update::{asks_about_automatic, takes_by_itself};
use linkunbound_core::{Strings, data_dir};
use linkunbound_shell::{HERE, Strip, takes_it_here};
use slint::ComponentHandle;

use super::{Ui, catalogue, open_settings, run_settings, store, ui};

const ASKING: &str = "ask";
const FAILED: &str = "failed";
const RELEASES: &str = "https://github.com/rgdevment/LinkUnbound/releases/latest";
const LOOK_ANSWERS_WITHIN: Duration = Duration::from_secs(3 * 60);
const PICKER_GOES_AWAY_WITHIN: Duration = Duration::from_secs(10 * 60);

fn question(words: &Strings) -> Strip {
    Strip {
        stage: ASKING,
        line: words.update_auto_ask.to_owned(),
        detail: String::new(),
        button: words.update_auto_yes.to_owned(),
        far: 0,
    }
}

/// A copy nobody has used yet is not asked in the version it was installed with.
fn settle_the_question_for_a_new_copy() {
    let prefs = store().prefs();
    let fresh =
        prefs.here_since.is_none() && store().rules().map_or(true, |set| set.rules.is_empty());
    if fresh && prefs.updates_asked_in.is_none() {
        let _ = store().edit_prefs(|kept| {
            kept.updates_asked_in = Some(HERE.to_owned());
            true
        });
    }
}

fn link_for(words: &Strings, stage: &str) -> &'static str {
    match stage {
        ASKING => words.update_auto_never,
        FAILED => words.update_download,
        _ => "",
    }
}

fn download_from_the_web() {
    let browsers = catalogue();
    let Some(to) =
        linkunbound_core::where_ours_opens(&browsers, store().rules().ok().as_ref(), RELEASES)
    else {
        return;
    };
    super::host::let_whoever_opens_next_come_forward();
    let _ = linkunbound_core::launch(
        &browsers,
        &to.browser_id,
        to.profile_id.as_deref(),
        to.private,
        RELEASES,
    );
}

fn take_by_itself() {
    if let Some(ui) = ui() {
        ui.take_by_itself();
    }
}

impl Ui {
    /// What the strip says now, read off the two files settings writes. An install that stopped
    /// reporting is one whose process died: told as failed, so the button comes back.
    pub(super) fn refresh_strip(self: &Rc<Self>) {
        use linkunbound_core::update::{gone_quiet, progress, progress_age, tell};
        let dir = data_dir();
        let looked = linkunbound_core::update::looked(&dir);
        let mut under_way = progress(&dir);
        if let Some(one) = under_way.as_mut()
            && gone_quiet(&one.stage, progress_age(&dir))
        {
            one.stage = "failed".to_owned();
            tell(&dir, one);
        }
        let words = self.words.get();
        let mut strip = linkunbound_shell::strip_for(
            &words,
            &looked,
            under_way.as_ref(),
            HERE,
            self.put_away.borrow().as_deref(),
        )
        .or_else(|| self.asking(&looked).then(|| question(&words)));
        let stage = strip.as_ref().map_or("", |one| one.stage);
        self.strip_stage.set(stage);
        let link = link_for(&words, stage);
        if !link.is_empty()
            && let Some(one) = strip.as_mut()
        {
            one.detail.clear();
        }
        linkunbound_shell::show_strip(&self.picker, strip.as_ref());
        self.picker.set_update_link(link.into());
        let busy = strip
            .as_ref()
            .is_some_and(|one| matches!(one.stage, "starting" | "getting" | "installing"));
        // An install the window started is followed here too: the file is the same.
        if busy && !self.progress_watch.running() {
            let ui = Rc::clone(self);
            self.progress_watch.start(
                slint::TimerMode::Repeated,
                Duration::from_millis(300),
                move || ui.refresh_strip(),
            );
        }
        if !busy {
            self.progress_watch.stop();
        }
    }

    /// Pressed: a Store copy is taken to settings, where the Store can be asked; any other copy
    /// has settings download and install with no window, and this watches the file it writes.
    fn take_update(self: &Rc<Self>) {
        use linkunbound_core::update::{Progress, looked, tell, under_way_for};
        let dir = data_dir();
        let looked = looked(&dir);
        let Some(found) = looked.found_version.clone() else {
            return;
        };
        // A press on top of a live install is one install; a route this copy cannot take by
        // itself — the Store's, a copy on a disk image, or one an older file never named — is
        // the window's to explain.
        if under_way_for(&dir, &found) {
            return;
        }
        if !linkunbound_shell::takes_it_here(&looked) {
            open_settings();
            return;
        }
        let started = Progress {
            version: found,
            stage: "starting".to_owned(),
            far: 0,
        };
        tell(&dir, &started);
        if !run_settings(&["--update"]) {
            tell(
                &dir,
                &Progress {
                    stage: "failed".to_owned(),
                    ..started
                },
            );
        }
        self.refresh_strip();
    }

    fn put_update_away(self: &Rc<Self>) {
        let dir = data_dir();
        let looked = linkunbound_core::update::looked(&dir);
        *self.put_away.borrow_mut() = looked.found_version;
        if linkunbound_core::update::progress(&dir).is_some_and(|one| one.stage == "failed") {
            linkunbound_core::update::settle(&dir);
        }
        self.refresh_strip();
    }

    /// Settings holds the network code; this only has to ask, within its interval, so a copy
    /// whose window is never opened still hears about a release.
    pub(super) fn keep_looking(&self) {
        // An install that was over when this copy started — it is the version now, or the file
        // is older than any offer — would otherwise show its last bar on the first picker.
        let dir = data_dir();
        if linkunbound_core::update::progress(&dir)
            .is_some_and(|one| !linkunbound_core::update::newer_than(&one.version, HERE))
        {
            linkunbound_core::update::settle(&dir);
        }
        settle_the_question_for_a_new_copy();
        // Read at each tick, not once: the switch in settings has to take effect without a
        // restart, like everything else there.
        let look = || {
            if store().prefs().looks_for_updates && run_settings(&["--look"]) {
                slint::Timer::single_shot(LOOK_ANSWERS_WITHIN, take_by_itself);
            }
        };
        self.looker.start(
            slint::TimerMode::Repeated,
            Duration::from_secs(6 * 60 * 60),
            look,
        );
        slint::Timer::single_shot(Duration::from_secs(20), look);
        // Once a session, off the start-up path: the icons of browsers long uninstalled are
        // what the cache otherwise fills with.
        slint::Timer::single_shot(Duration::from_secs(90), || {
            let browsers = catalogue();
            linkunbound_core::prune_icons(
                &data_dir().join("icons"),
                browsers.iter().map(|b| b.id.as_str()),
            );
        });
    }

    fn asking(&self, looked: &linkunbound_core::update::Looked) -> bool {
        asks_about_automatic(&store().prefs(), HERE, takes_it_here(looked))
    }

    pub(super) fn strip_pressed(self: &Rc<Self>) {
        if self.strip_stage.get() == ASKING {
            self.answer(true);
        } else {
            self.take_update();
        }
    }

    pub(super) fn strip_closed(self: &Rc<Self>) {
        if self.strip_stage.get() == ASKING {
            let _ = store().edit_prefs(|kept| {
                kept.updates_asked_in = Some(HERE.to_owned());
                true
            });
            self.refresh_strip();
        } else {
            self.put_update_away();
        }
    }

    pub(super) fn strip_link_pressed(self: &Rc<Self>) {
        match self.strip_stage.get() {
            ASKING => self.answer(false),
            FAILED => download_from_the_web(),
            _ => {}
        }
    }

    fn answer(self: &Rc<Self>, automatically: bool) {
        let _ = store().edit_prefs(|kept| {
            kept.updates_automatically = Some(automatically);
            true
        });
        self.refresh_strip();
    }

    /// Never under the person's hands: the installer closes the picker along with the rest.
    fn take_by_itself(self: &Rc<Self>) {
        use linkunbound_core::update::{looked, progress};
        let dir = data_dir();
        let looked = looked(&dir);
        let attempt = progress(&dir);
        if !takes_by_itself(&store().prefs(), &looked, attempt.as_ref(), HERE)
            || !takes_it_here(&looked)
        {
            return;
        }
        if self.picker.window().is_visible() {
            slint::Timer::single_shot(PICKER_GOES_AWAY_WITHIN, take_by_itself);
            return;
        }
        self.take_update();
    }
}

#[cfg(test)]
mod tests {
    use super::{ASKING, FAILED, link_for, question};
    use linkunbound_core::Language;

    #[test]
    fn the_question_is_a_strip_in_the_ask_stage_with_the_words_of_each_language() {
        for language in [Language::Spanish, Language::English] {
            let words = language.strings();
            let strip = question(&words);
            assert_eq!(strip.stage, ASKING);
            assert_eq!(strip.line, words.update_auto_ask);
            assert!(strip.detail.is_empty(), "the second line is the never link");
            assert_eq!(strip.button, words.update_auto_yes);
            assert_eq!(strip.far, 0);
        }
        assert_ne!(
            question(&Language::Spanish.strings()).line,
            question(&Language::English.strings()).line
        );
    }

    #[test]
    fn the_strip_link_says_never_for_the_question_and_download_for_a_failure_in_each_language() {
        for language in [Language::Spanish, Language::English] {
            let words = language.strings();
            assert_eq!(link_for(&words, ASKING), words.update_auto_never);
            assert_eq!(link_for(&words, FAILED), words.update_download);
            for stage in ["", "starting", "getting", "installing", "ready"] {
                assert_eq!(link_for(&words, stage), "", "stage {stage:?} has no link");
            }
        }
        assert_eq!(
            link_for(&Language::Spanish.strings(), FAILED),
            "Descargar desde la web"
        );
        assert_eq!(
            link_for(&Language::English.strings(), FAILED),
            "Download from the web"
        );
    }
}
