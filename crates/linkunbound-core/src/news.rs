use semver::Version;

/// The versions whose news this copy has not told yet, newest first.
#[must_use]
pub fn untold(fresh: bool, seen: Option<&str>, running: &str, told: &[String]) -> Vec<String> {
    let Ok(now) = Version::parse(running) else {
        return Vec::new();
    };
    // A first install has nothing to compare with, and a candidate's notes are its release page.
    if fresh || !now.pre.is_empty() {
        return Vec::new();
    }
    let since = seen.and_then(|one| Version::parse(one.trim()).ok());
    if since.as_ref().is_some_and(|since| *since >= now) {
        return Vec::new();
    }
    let parsed = || {
        told.iter()
            .filter_map(|said| Version::parse(said).ok().map(|one| (one, said)))
    };
    let mut news: Vec<(Version, &String)> = match &since {
        Some(since) => parsed()
            .filter(|(one, _)| *one <= now && one > since)
            .collect(),
        // Nothing says which version came before, so only the one it runs is surely new.
        None => parsed()
            .filter(|(one, _)| *one == now || is_minor_of(one, &now))
            .collect(),
    };
    news.sort_by(|a, b| b.0.cmp(&a.0));
    news.dedup_by(|a, b| a.0 == b.0);
    news.into_iter().map(|(_, said)| said.clone()).collect()
}

#[must_use]
pub fn behind(seen: Option<&str>, running: &str) -> bool {
    let Ok(now) = Version::parse(running) else {
        return false;
    };
    seen.and_then(|one| Version::parse(one.trim()).ok())
        .is_none_or(|since| since < now)
}

fn is_minor_of(one: &Version, patch: &Version) -> bool {
    patch.patch > 0 && patch.pre.is_empty() && *one == Version::new(patch.major, patch.minor, 0)
}

#[cfg(test)]
mod tests {
    use super::{behind, untold};

    fn told(list: &[&str]) -> Vec<String> {
        list.iter().map(|one| (*one).to_owned()).collect()
    }

    #[test]
    fn a_first_install_is_told_nothing() {
        assert!(untold(true, None, "2.6.0", &told(&["2.6.0"])).is_empty());
    }

    #[test]
    fn an_update_tells_what_came_since_the_last_one_told_newest_first() {
        let all = told(&["2.4.0", "2.5.0", "2.6.0", "2.7.0"]);
        assert_eq!(
            untold(false, Some("2.4.0"), "2.6.0", &all),
            told(&["2.6.0", "2.5.0"])
        );
    }

    #[test]
    fn a_copy_told_nothing_before_hears_only_what_it_runs() {
        let all = told(&["2.5.0", "2.6.0", "2.7.0"]);
        assert_eq!(untold(false, None, "2.6.0", &all), told(&["2.6.0"]));
        assert_eq!(untold(false, None, "2.6.2", &all), told(&["2.6.0"]));
    }

    #[test]
    fn the_same_version_is_not_told_twice() {
        assert!(untold(false, Some("2.6.0"), "2.6.0", &told(&["2.6.0"])).is_empty());
        assert!(untold(false, Some("2.7.0"), "2.6.0", &told(&["2.6.0"])).is_empty());
    }

    #[test]
    fn a_patch_tells_its_minor_release_only_to_whoever_missed_it() {
        let all = told(&["2.5.0", "2.6.0"]);
        assert_eq!(
            untold(false, Some("2.5.0"), "2.6.1", &all),
            told(&["2.6.0"])
        );
        assert!(untold(false, Some("2.6.0"), "2.6.1", &all).is_empty());
        assert!(untold(false, Some("2.6.0"), "2.6.1-rc.1", &all).is_empty());
    }

    #[test]
    fn a_candidate_and_a_build_without_a_version_tell_nothing() {
        let all = told(&["2.6.0"]);
        assert!(untold(false, Some("2.5.0"), "2.6.0-rc.1", &all).is_empty());
        assert!(untold(false, None, "2.6.0-rc.1", &told(&["2.5.0", "2.6.0"])).is_empty());
        assert!(untold(false, Some("2.5.0"), "dev", &all).is_empty());
    }

    #[test]
    fn behind_is_true_until_the_running_version_is_written_down() {
        assert!(behind(None, "2.6.0"));
        assert!(behind(Some("2.5.0"), "2.6.0"));
        assert!(behind(Some("nonsense"), "2.6.0"));
        assert!(!behind(Some("2.6.0"), "2.6.0"));
        assert!(!behind(Some("2.5.0"), "dev"));
    }
}
