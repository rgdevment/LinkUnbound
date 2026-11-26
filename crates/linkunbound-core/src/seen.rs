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
    fn an_unreadable_file_is_nothing_seen() {
        let dir = std::env::temp_dir().join(format!("linkunbound-seen-bad-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("a scratch directory");
        std::fs::write(dir.join(APPS_SEEN), "not json").expect("written");
        assert!(Store::at(&dir).apps_seen().is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
