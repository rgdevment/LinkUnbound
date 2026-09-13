use std::path::{Path, PathBuf};

pub const RESIDENT: &str = "linkunbound-shell";
pub const SETTINGS: &str = "linkunbound-settings";

fn named(beside: &Path, stem: &str) -> PathBuf {
    let file = if cfg!(windows) {
        format!("{stem}.exe")
    } else {
        stem.to_owned()
    };
    beside.with_file_name(file)
}

/// The binary the shell must invoke with a link. Registering anything else —
/// the settings window, most of all — leaves Windows handing links to a process
/// that does not know what to do with one.
#[must_use]
pub fn link_handler(running: &Path) -> PathBuf {
    named(running, RESIDENT)
}

#[must_use]
pub fn settings_binary(running: &Path) -> PathBuf {
    named(running, SETTINGS)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Registered {
    /// The command names the resident, at the place it is running from.
    Correct,
    /// Registered, but the command names something else entirely.
    Elsewhere(String),
    /// Registered to our own settings binary: the failure this check exists for.
    WrongBinary(String),
    Absent,
}

/// Reads a registered command and says what it really points at. The comparison
/// is by path, not by "is it us": a second install, a leftover from a move, or
/// the wrong binary of this very build all read as registered otherwise.
#[must_use]
pub fn what_is_registered(command: Option<&str>, expected: &Path) -> Registered {
    let Some(command) = command else {
        return Registered::Absent;
    };
    let command = command.trim();
    let path = match command.strip_prefix('"') {
        Some(quoted) => quoted.split('"').next().unwrap_or(quoted),
        None => command.strip_suffix("%1").unwrap_or(command).trim(),
    };
    if path.is_empty() {
        return Registered::Absent;
    }
    let tidy = |p: &str| p.replace('/', "\\").to_ascii_lowercase();

    if tidy(path) == tidy(&expected.to_string_lossy()) {
        return Registered::Correct;
    }
    if tidy(path) == tidy(&settings_binary(expected).to_string_lossy()) {
        return Registered::WrongBinary(path.to_owned());
    }
    Registered::Elsewhere(path.to_owned())
}

#[cfg(test)]
mod tests {
    #[cfg(windows)]
    use super::settings_binary;
    use super::{Registered, link_handler, what_is_registered};
    use std::path::{Path, PathBuf};

    fn at(dir: &str) -> PathBuf {
        Path::new(dir).join(if cfg!(windows) {
            "linkunbound-shell.exe"
        } else {
            "linkunbound-shell"
        })
    }

    #[test]
    fn the_handler_is_the_resident_whichever_binary_asks() {
        let from_settings = Path::new(r"C:\Program Files\LinkUnbound\linkunbound-settings.exe");
        let from_resident = Path::new(r"C:\Program Files\LinkUnbound\linkunbound-shell.exe");
        assert_eq!(link_handler(from_settings), link_handler(from_resident));
        assert!(
            link_handler(from_settings)
                .to_string_lossy()
                .contains("shell")
        );
    }

    #[cfg(windows)]
    #[test]
    fn our_own_settings_binary_is_named_as_the_wrong_one() {
        let expected = at(r"C:\Program Files\LinkUnbound");
        let wrong = settings_binary(&expected).to_string_lossy().into_owned();
        assert_eq!(
            what_is_registered(Some(&format!("\"{wrong}\" \"%1\"")), &expected),
            Registered::WrongBinary(wrong)
        );
    }

    #[cfg(windows)]
    #[test]
    fn a_build_tree_left_over_from_debugging_is_not_mistaken_for_ours() {
        let installed = at(r"C:\Program Files\LinkUnbound");
        let debug = at(r"D:\Code\LinkUnbound\target\debug");
        let command = format!("\"{}\" \"%1\"", debug.to_string_lossy());
        assert_eq!(
            what_is_registered(Some(&command), &installed),
            Registered::Elsewhere(debug.to_string_lossy().into_owned())
        );
    }

    #[cfg(windows)]
    #[test]
    fn the_right_command_reads_as_correct_however_it_is_spelled() {
        let expected = at(r"C:\Program Files\LinkUnbound");
        for spelling in [
            format!("\"{}\" \"%1\"", expected.to_string_lossy()),
            format!("{} %1", expected.to_string_lossy()),
            format!(
                "\"{}\" \"%1\"",
                expected.to_string_lossy().to_ascii_uppercase()
            ),
            format!(
                "\"{}\" \"%1\"",
                expected.to_string_lossy().replace('\\', "/")
            ),
        ] {
            assert_eq!(
                what_is_registered(Some(&spelling), &expected),
                Registered::Correct,
                "{spelling}"
            );
        }
    }

    #[test]
    fn nothing_registered_is_not_read_as_something() {
        assert_eq!(
            what_is_registered(None, &at("/opt/linkunbound")),
            Registered::Absent
        );
    }
}
