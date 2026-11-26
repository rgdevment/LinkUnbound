use serde::{Deserialize, Serialize};

use crate::{Origin, Store, seconds_now};

pub const APPS_SEEN: &str = "apps-seen.json";
const KEPT: usize = 50;

/// Which app links came from and when, never a link; the diagnostic report leaves it out.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeenApp {
    pub key: String,
    pub label: String,
    pub links: u64,
    pub last_seen: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub program: Option<String>,
}

impl Store {
    #[must_use]
    pub fn apps_seen(&self) -> Vec<SeenApp> {
        std::fs::read(self.dir().join(APPS_SEEN))
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default()
    }

    /// Only the resident writes it, and a failure costs a line in a list, never the link.
    pub fn note_app_seen(&self, origin: &Origin, program: Option<&str>) {
        let Some(now) = seconds_now() else { return };
        let seen = with_sighting(self.apps_seen(), origin, program, now);
        let Ok(body) = serde_json::to_string(&seen) else {
            return;
        };
        let dir = self.dir();
        let _ = std::fs::create_dir_all(dir);
        let staged = dir.join(format!("{APPS_SEEN}.{}.tmp", std::process::id()));
        if std::fs::write(&staged, body).is_err()
            || std::fs::rename(&staged, dir.join(APPS_SEEN)).is_err()
        {
            let _ = std::fs::remove_file(&staged);
        }
    }

    pub fn forget_apps_seen(&self) {
        let _ = std::fs::remove_file(self.dir().join(APPS_SEEN));
    }
}

/// Behind the picker the program in front is the picker, so an unknown one keeps the last known.
fn with_sighting(
    mut seen: Vec<SeenApp>,
    origin: &Origin,
    program: Option<&str>,
    now: u64,
) -> Vec<SeenApp> {
    let before = seen
        .iter()
        .position(|app| app.key == origin.key)
        .map(|at| seen.remove(at));
    let links = before.as_ref().map_or(0, |app| app.links).saturating_add(1);
    let program = program
        .map(str::to_owned)
        .or_else(|| before.and_then(|app| app.program));
    seen.insert(
        0,
        SeenApp {
            key: origin.key.clone(),
            label: origin.label.clone(),
            links,
            last_seen: now,
            program,
        },
    );
    seen.truncate(KEPT);
    seen
}

#[cfg(test)]
mod tests {
    use super::{APPS_SEEN, KEPT, SeenApp, with_sighting};
    use crate::{Origin, Store};

    fn app(key: &str, links: u64, last_seen: u64) -> SeenApp {
        SeenApp {
            key: key.to_owned(),
            label: key.to_owned(),
            links,
            last_seen,
            program: None,
        }
    }

    #[test]
    fn a_new_app_comes_first_with_one_link() {
        let seen = with_sighting(vec![app("slack", 3, 10)], &Origin::named("olk"), None, 20);
        assert_eq!(seen, vec![app("olk", 1, 20), app("slack", 3, 10)]);
    }

    #[test]
    fn an_app_seen_again_moves_to_the_front_with_one_more_link() {
        let seen = with_sighting(
            vec![app("olk", 1, 20), app("slack", 3, 10)],
            &Origin::named("slack"),
            None,
            30,
        );
        assert_eq!(seen, vec![app("slack", 4, 30), app("olk", 1, 20)]);
    }

    #[test]
    fn the_program_is_replaced_when_known_and_kept_when_not() {
        let known = with_sighting(
            Vec::new(),
            &Origin::named("slack"),
            Some("C:/a/slack.exe"),
            1,
        );
        let moved = with_sighting(known, &Origin::named("slack"), Some("C:/b/slack.exe"), 2);
        assert_eq!(moved[0].program.as_deref(), Some("C:/b/slack.exe"));
        let behind = with_sighting(moved, &Origin::named("slack"), None, 3);
        assert_eq!(behind[0].program.as_deref(), Some("C:/b/slack.exe"));
    }

    #[test]
    fn a_mac_app_keeps_its_bundle_id_and_takes_the_name_it_goes_by_now() {
        let before = SeenApp {
            key: "com.tinyspeck.slackmacgap".to_owned(),
            label: "slack".to_owned(),
            links: 2,
            last_seen: 1,
            program: None,
        };
        let renamed = Origin {
            key: "com.tinyspeck.slackmacgap".to_owned(),
            label: "slack beta".to_owned(),
        };
        let seen = with_sighting(vec![before], &renamed, None, 5);
        assert_eq!(seen.len(), 1);
        assert_eq!(seen[0].label, "slack beta");
        assert_eq!(seen[0].links, 3);
    }

    #[test]
    fn past_the_ceiling_the_oldest_goes() {
        let full: Vec<SeenApp> = (0..KEPT as u64)
            .map(|i| app(&format!("app{i}"), 1, 100 - i))
            .collect();
        let seen = with_sighting(full, &Origin::named("new"), None, 200);
        assert_eq!(seen.len(), KEPT);
        assert_eq!(seen[0].key, "new");
        assert!(seen.iter().all(|a| a.key != format!("app{}", KEPT - 1)));
    }

    #[test]
    fn what_is_noted_is_read_back_and_forgotten_on_request() {
        let dir = std::env::temp_dir().join(format!("linkunbound-seen-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let store = Store::at(&dir);
        assert!(store.apps_seen().is_empty());
        store.note_app_seen(&Origin::named("slack"), Some("C:/slack.exe"));
        store.note_app_seen(&Origin::named("slack"), None);
        let seen = store.apps_seen();
        assert_eq!(seen.len(), 1);
        assert_eq!(seen[0].links, 2);
        assert_eq!(seen[0].program.as_deref(), Some("C:/slack.exe"));
        store.forget_apps_seen();
        assert!(store.apps_seen().is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn below_the_ceiling_nothing_goes_and_at_it_only_a_new_app_pushes_one_out() {
        let almost: Vec<SeenApp> = (0..KEPT as u64 - 1)
            .map(|i| app(&format!("app{i}"), 1, 100 - i))
            .collect();
        let full = with_sighting(almost, &Origin::named("new"), None, 200);
        assert_eq!(full.len(), KEPT);
        assert!(full.iter().any(|a| a.key == format!("app{}", KEPT - 2)));
        let again = with_sighting(full, &Origin::named("app3"), None, 300);
        assert_eq!(again.len(), KEPT, "an app already there pushes nobody out");
        assert_eq!(again[0].key, "app3");
        assert!(again.iter().any(|a| a.key == format!("app{}", KEPT - 2)));
        let past = with_sighting(again, &Origin::named("newer"), None, 400);
        assert_eq!(past.len(), KEPT);
        assert!(past.iter().all(|a| a.key != format!("app{}", KEPT - 2)));
    }

    #[test]
    fn the_count_goes_up_by_one_each_time_and_the_date_is_the_latest() {
        let first = with_sighting(Vec::new(), &Origin::named("slack"), None, 1);
        let second = with_sighting(first, &Origin::named("slack"), None, 2);
        let third = with_sighting(second, &Origin::named("slack"), None, 9);
        assert_eq!((third[0].links, third[0].last_seen), (3, 9));
        let huge = vec![app("slack", u64::MAX, 1)];
        let capped = with_sighting(huge, &Origin::named("slack"), None, 2);
        assert_eq!(capped[0].links, u64::MAX);
    }

    #[test]
    fn an_app_gets_a_program_only_from_a_sighting_that_has_one() {
        let none = with_sighting(Vec::new(), &Origin::named("slack"), None, 1);
        assert!(none[0].program.is_none());
        let some = with_sighting(none, &Origin::named("slack"), Some("C:/s.exe"), 2);
        assert_eq!(some[0].program.as_deref(), Some("C:/s.exe"));
        let other = with_sighting(some, &Origin::named("olk"), None, 3);
        assert!(other[0].program.is_none());
        assert_eq!(other[1].program.as_deref(), Some("C:/s.exe"));
    }

    #[test]
    fn the_file_holds_the_app_and_when_but_no_link_and_leaves_nothing_staged() {
        let dir =
            std::env::temp_dir().join(format!("linkunbound-seen-file-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let store = Store::at(&dir);
        store.note_app_seen(&Origin::named("slack"), None);
        let body = std::fs::read_to_string(dir.join(APPS_SEEN)).expect("written");
        assert!(
            !body.contains("program"),
            "an unknown program is left out: {body}"
        );
        assert!(!body.contains("http"));
        assert_eq!(std::fs::read_dir(&dir).expect("a directory").count(), 1);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn an_unreadable_file_is_nothing_seen() {
        let dir = std::env::temp_dir().join(format!("linkunbound-seen-bad-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("a scratch directory");
        std::fs::write(dir.join(APPS_SEEN), "not json").expect("written");
        assert!(Store::at(&dir).apps_seen().is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
