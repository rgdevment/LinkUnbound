use linkunbound_core::{Origin, Store};

/// Noted before the rules are asked, so an app a rule always answers is listed as well.
#[must_use]
pub fn noted(store: &Store, source: Option<Origin>) -> Option<Origin> {
    if let Some(origin) = &source {
        store.note_app_seen(origin, program_of(origin).as_deref());
    }
    source
}

#[cfg(windows)]
fn program_of(origin: &Origin) -> Option<String> {
    let front = linkunbound_win::foreground_process_path()?;
    let stem = std::path::Path::new(&front)
        .file_stem()?
        .to_str()?
        .to_lowercase();
    (stem == origin.key).then_some(front)
}

#[cfg(not(windows))]
fn program_of(_origin: &Origin) -> Option<String> {
    None
}

#[cfg(test)]
mod tests {
    use super::noted;
    use linkunbound_core::{Origin, Store};

    #[test]
    fn the_app_a_link_came_from_is_noted_and_handed_on_unchanged() {
        let dir = std::env::temp_dir().join(format!("linkunbound-noted-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let store = Store::at(&dir);
        assert_eq!(noted(&store, None), None);
        assert!(store.apps_seen().is_empty());
        let origin = Origin::named("no-such-app-in-front");
        assert_eq!(noted(&store, Some(origin.clone())), Some(origin));
        let seen = store.apps_seen();
        assert_eq!(seen.len(), 1);
        assert_eq!(seen[0].key, "no-such-app-in-front");
        assert_eq!(
            seen[0].program, None,
            "only the program in front is its program"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}
