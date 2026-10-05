use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::report::redact;

pub const JOURNAL: &str = "errors.log";
const KEPT: usize = 200;
const GONE: &str = "[…]";
const BOUNDS: &str = "\\/\"':(),;";

pub fn note(dir: &Path, area: &'static str, what: &str) {
    let homes: Vec<String> = ["USERPROFILE", "HOME"]
        .iter()
        .filter_map(|key| std::env::var(key).ok())
        .collect();
    let line = format!(
        "{} [{area}] {}",
        stamp(SystemTime::now()),
        scrubbed(what, &homes)
    );
    let mut lines = journal(dir);
    lines.push(line);
    let from = lines.len().saturating_sub(KEPT);
    let _ = std::fs::create_dir_all(dir);
    let staged = dir.join(format!("{JOURNAL}.{}.tmp", std::process::id()));
    if std::fs::write(&staged, lines[from..].join("\n") + "\n").is_ok()
        && std::fs::rename(&staged, dir.join(JOURNAL)).is_err()
    {
        let _ = std::fs::remove_file(&staged);
    }
}

#[must_use]
pub fn journal(dir: &Path) -> Vec<String> {
    std::fs::read(dir.join(JOURNAL))
        .map(|bytes| {
            String::from_utf8_lossy(&bytes)
                .lines()
                .filter(|line| !line.trim().is_empty())
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

#[must_use]
pub fn scrubbed(text: &str, homes: &[String]) -> String {
    let mut flat = text.replace(['\r', '\n'], " ");
    let mut homes: Vec<&String> = homes.iter().filter(|home| home.len() > 3).collect();
    homes.sort_by_key(|home| std::cmp::Reverse(home.len()));
    for home in homes {
        flat = replaced_ignoring_case(&flat, home, "~");
        flat = replaced_ignoring_case(&flat, &home.replace('\\', "/"), "~");
    }
    for marker in [r"\users\", "/users/", "/home/"] {
        flat = owner_hidden(&flat, marker);
    }
    flat.split(' ')
        .map(|word| {
            let bare = word.trim_matches(|c: char| "\"'()<>[],;`".contains(c));
            if bare.contains("://") {
                word.replace(bare, &redact(bare, GONE))
            } else if looks_like_an_address(bare) {
                word.replace(bare, GONE)
            } else {
                word.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn looks_like_an_address(word: &str) -> bool {
    if word.to_ascii_lowercase().starts_with("www.") {
        return true;
    }
    let Some((scheme, rest)) = word.split_once(':') else {
        return false;
    };
    scheme.len() > 1
        && scheme.starts_with(|c: char| c.is_ascii_alphabetic())
        && scheme
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "+.-".contains(c))
        && rest.chars().next().is_some_and(|c| !"\\/: ".contains(c))
}

fn owner_hidden(text: &str, marker: &str) -> String {
    let lower = text.to_ascii_lowercase();
    let mut out = String::with_capacity(text.len());
    let mut at = 0;
    while let Some(found) = lower[at..].find(marker) {
        let after = at + found + marker.len();
        out.push_str(&text[at..after]);
        let end = text[after..]
            .find(|c: char| BOUNDS.contains(c))
            .map_or(text.len(), |end| after + end);
        let owner = after + text[after..end].trim_end().len();
        if owner > after {
            out.push('…');
        }
        at = owner;
    }
    out.push_str(&text[at..]);
    out
}

fn replaced_ignoring_case(text: &str, what: &str, with: &str) -> String {
    let lower = text.to_ascii_lowercase();
    let wanted = what.to_ascii_lowercase();
    let mut out = String::with_capacity(text.len());
    let mut at = 0;
    while let Some(found) = lower[at..].find(&wanted) {
        let end = at + found + wanted.len();
        let whole = text[end..]
            .chars()
            .next()
            .is_none_or(|c| c.is_whitespace() || BOUNDS.contains(c));
        out.push_str(&text[at..at + found]);
        out.push_str(if whole { with } else { &text[at + found..end] });
        at = end;
    }
    out.push_str(&text[at..]);
    out
}

fn stamp(when: SystemTime) -> String {
    let seconds = when
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_secs());
    let days = i64::try_from(seconds / 86_400).unwrap_or(0);
    let rest = seconds % 86_400;
    let (year, month, day) = civil(days);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        rest / 3600,
        rest % 3600 / 60,
        rest % 60
    )
}

fn civil(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = u32::try_from(doy - (153 * mp + 2) / 5 + 1).unwrap_or(1);
    let month = u32::try_from(if mp < 10 { mp + 3 } else { mp - 9 }).unwrap_or(1);
    let year = yoe + era * 400 + i64::from(month <= 2);
    (year, month, day)
}

#[cfg(test)]
mod tests {
    use super::{KEPT, journal, note, scrubbed, stamp};
    use std::time::{Duration, UNIX_EPOCH};

    fn ana() -> Vec<String> {
        vec![r"C:\Users\Ana".to_owned()]
    }

    #[test]
    fn a_link_keeps_only_its_host_and_the_home_folder_loses_its_owner() {
        let said = scrubbed(
            r#"launch of "C:\Users\Ana\AppData\Local\Chrome\chrome.exe" for https://intranet.corp/sueldos?id=4 refused"#,
            &ana(),
        );
        assert!(!said.contains("Ana"), "{said}");
        assert!(!said.contains("sueldos"), "{said}");
        assert!(
            said.contains(r"~\AppData\Local\Chrome\chrome.exe"),
            "{said}"
        );
        assert!(said.contains("https://intranet.corp/…"), "{said}");
    }

    #[test]
    fn the_home_folder_is_found_whatever_its_case_or_slashes() {
        assert_eq!(
            scrubbed(r"c:\users\ANA\x.json unreadable", &ana()),
            r"~\x.json unreadable"
        );
        assert_eq!(
            scrubbed("C:/Users/Ana/x.json unreadable", &ana()),
            "~/x.json unreadable"
        );
    }

    #[test]
    fn an_owner_the_home_does_not_name_is_hidden_all_the_same() {
        for (path, kept) in [
            (r"D:\Users\Bea\rules.json", r"D:\Users\…\rules.json"),
            (r"C:\Users\BEA~1\rules.json", r"C:\Users\…\rules.json"),
            ("/Users/bea/Library/x", "/Users/…/Library/x"),
            ("/home/bea/.config/x", "/home/…/.config/x"),
            (r"D:\Users\Ana Maria\rules.json", r"D:\Users\…\rules.json"),
            (r#""C:\Users\Ana Maria" missing"#, r#""C:\Users\…" missing"#),
            (
                r"cannot create C:\Users\Bea (os error 5)",
                r"cannot create C:\Users\… (os error 5)",
            ),
            (r"C:\Users\Bea: access denied", r"C:\Users\…: access denied"),
        ] {
            assert_eq!(scrubbed(path, &[]), kept);
        }
    }

    #[test]
    fn an_address_without_slashes_is_cut_too() {
        let said = scrubbed(
            "open mailto:ana@corp.com and www.x.com/secret?a=1 and file:C:/x failed",
            &[],
        );
        assert!(!said.contains("ana@corp.com"), "{said}");
        assert!(!said.contains("secret"), "{said}");
        assert!(!said.contains("file:C"), "{said}");
        assert_eq!(
            scrubbed(r"firefox: the browser would not start (C:\x)", &[]),
            r"firefox: the browser would not start (C:\x)"
        );
    }

    #[test]
    fn another_account_that_starts_like_the_home_is_not_taken_for_it() {
        assert_eq!(
            scrubbed(r"C:\Users\Anabel\rules.json unreadable", &ana()),
            r"C:\Users\…\rules.json unreadable"
        );
        assert_eq!(scrubbed(r"C:\Users\Ana unreadable", &ana()), "~ unreadable");
    }

    #[test]
    fn the_tilde_stands_for_the_home_folder_and_nothing_else() {
        let said = scrubbed(r"C:\Users\Ana\AppData\Local\LinkUnbound\rules.json", &ana());
        assert_eq!(said, r"~\AppData\Local\LinkUnbound\rules.json");
    }

    #[test]
    fn a_line_break_cannot_forge_another_entry() {
        let said = scrubbed("first\n2026-01-01T00:00:00Z [rules] forged", &[]);
        assert!(!said.contains('\n'));
    }

    #[test]
    fn dates_are_written_in_utc_the_way_people_read_them() {
        assert_eq!(stamp(UNIX_EPOCH), "1970-01-01T00:00:00Z");
        assert_eq!(
            stamp(UNIX_EPOCH + Duration::from_secs(1_791_158_400 + 3_661)),
            "2026-10-05T01:01:01Z"
        );
        assert_eq!(
            stamp(UNIX_EPOCH + Duration::from_secs(951_782_400)),
            "2000-02-29T00:00:00Z"
        );
    }

    #[test]
    fn the_journal_keeps_only_its_latest_entries() {
        let dir = std::env::temp_dir().join(format!("linkunbound-journal-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        for at in 0..KEPT + 5 {
            note(&dir, "test", &format!("entry {at}"));
        }
        let kept = journal(&dir);
        assert_eq!(kept.len(), KEPT);
        assert!(kept[0].ends_with("entry 5"));
        assert!(kept[KEPT - 1].ends_with(&format!("entry {}", KEPT + 4)));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
