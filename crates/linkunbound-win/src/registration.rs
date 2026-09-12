use winreg::RegKey;
use winreg::enums::{HKEY_CURRENT_USER, KEY_READ, KEY_WRITE};

use crate::RegistrationError;

pub const PROG_ID: &str = "LinkUnboundURL";
const APP_NAME: &str = "LinkUnbound";
const APP_DESCRIPTION: &str = "Browser picker for Windows";

const URL_SCHEMES: [&str; 2] = ["http", "https"];

const FILE_EXTENSIONS: [&str; 10] = [
    ".htm", ".html", ".xhtml", ".xht", ".pdf", ".svg", ".mhtml", ".mht", ".shtml", ".webp",
];

const USER_CHOICE_PATHS: [&str; 7] = [
    r"Software\Microsoft\Windows\Shell\Associations\UrlAssociations\http\UserChoice",
    r"Software\Microsoft\Windows\Shell\Associations\UrlAssociations\https\UserChoice",
    r"Software\Microsoft\Windows\CurrentVersion\Explorer\FileExts\.htm\UserChoice",
    r"Software\Microsoft\Windows\CurrentVersion\Explorer\FileExts\.html\UserChoice",
    r"Software\Microsoft\Windows\CurrentVersion\Explorer\FileExts\.xhtml\UserChoice",
    r"Software\Microsoft\Windows\CurrentVersion\Explorer\FileExts\.svg\UserChoice",
    r"Software\Microsoft\Windows\CurrentVersion\Explorer\FileExts\.pdf\UserChoice",
];

/// Every path is relative to a root so tests can write somewhere disposable
/// instead of the live association keys.
#[derive(Debug, Clone)]
pub struct Registration {
    root: String,
}

impl Default for Registration {
    fn default() -> Self {
        Self {
            root: "Software".to_owned(),
        }
    }
}

/// A build tree must never own the registration. Its path disappears as soon as
/// the tree is cleaned or rebuilt elsewhere, and since `HKCU\Software\Classes`
/// shadows `HKLM`, the dead ProgId left behind hijacks links from the installed
/// copy — Store or standalone alike. By path component, because `--target`
/// nests the profile deeper and a custom profile spells neither name.
#[must_use]
pub fn is_build_tree(exe: &str) -> bool {
    let path = exe.replace('/', "\\").to_ascii_lowercase();
    path.split('\\')
        .rev()
        .skip(1)
        .any(|component| component == "target")
}

fn quoted(exe: &str) -> String {
    format!("\"{}\"", exe.replace('/', "\\"))
}

impl Registration {
    #[must_use]
    pub fn under(root: &str) -> Self {
        Self {
            root: root.to_owned(),
        }
    }

    fn classes(&self) -> String {
        format!(r"{}\Classes", self.root)
    }

    fn hkcu() -> RegKey {
        RegKey::predef(HKEY_CURRENT_USER)
    }

    fn write_prog_id(&self, exe: &str) -> Result<(), RegistrationError> {
        let quoted_exe = quoted(exe);
        let (prog, _) = Self::hkcu().create_subkey(format!(r"{}\{PROG_ID}", self.classes()))?;
        prog.set_value("", &"LinkUnbound URL")?;
        prog.set_value("FriendlyTypeName", &"LinkUnbound URL")?;
        prog.set_value("EditFlags", &2u32)?;

        let (app, _) = prog.create_subkey("Application")?;
        app.set_value("ApplicationName", &APP_NAME)?;
        app.set_value("ApplicationDescription", &APP_DESCRIPTION)?;
        app.set_value("ApplicationIcon", &format!("{quoted_exe},0"))?;

        let (icon, _) = prog.create_subkey("DefaultIcon")?;
        icon.set_value("", &format!("{quoted_exe},0"))?;

        let (command, _) = prog.create_subkey(r"shell\open\command")?;
        command.set_value("", &format!("{quoted_exe} \"%1\""))?;
        Ok(())
    }

    fn write_start_menu(&self, exe: &str) -> Result<(), RegistrationError> {
        let quoted_exe = quoted(exe);
        let path = format!(r"{}\Clients\StartMenuInternet\{APP_NAME}", self.root);
        let (key, _) = Self::hkcu().create_subkey(path)?;
        key.set_value("", &APP_NAME)?;

        let (icon, _) = key.create_subkey("DefaultIcon")?;
        icon.set_value("", &format!("{quoted_exe},0"))?;

        let (command, _) = key.create_subkey(r"shell\open\command")?;
        command.set_value("", &quoted_exe)?;

        let (install, _) = key.create_subkey("InstallInfo")?;
        install.set_value("ReinstallCommand", &quoted_exe)?;
        install.set_value("IconsVisible", &1u32)?;
        Ok(())
    }

    fn write_capabilities(&self, exe: &str) -> Result<(), RegistrationError> {
        let quoted_exe = quoted(exe);
        let (caps, _) =
            Self::hkcu().create_subkey(format!(r"{}\{APP_NAME}\Capabilities", self.root))?;
        caps.set_value("ApplicationName", &APP_NAME)?;
        caps.set_value("ApplicationDescription", &APP_DESCRIPTION)?;
        caps.set_value("ApplicationIcon", &format!("{quoted_exe},0"))?;

        let (start, _) = caps.create_subkey("Startmenu")?;
        start.set_value("StartMenuInternet", &APP_NAME)?;

        let (urls, _) = caps.create_subkey("URLAssociations")?;
        for scheme in URL_SCHEMES {
            urls.set_value(scheme, &PROG_ID)?;
        }

        let (files, _) = caps.create_subkey("FileAssociations")?;
        for ext in FILE_EXTENSIONS {
            files.set_value(ext, &PROG_ID)?;
        }
        Ok(())
    }

    fn write_registered_applications(&self) -> Result<(), RegistrationError> {
        let (key, _) =
            Self::hkcu().create_subkey(format!(r"{}\RegisteredApplications", self.root))?;
        key.set_value(APP_NAME, &format!(r"{}\{APP_NAME}\Capabilities", self.root))?;
        Ok(())
    }

    fn write_open_with(&self) -> Result<(), RegistrationError> {
        for ext in FILE_EXTENSIONS {
            let path = format!(r"{}\{ext}\OpenWithProgIds", self.classes());
            let (key, _) = Self::hkcu().create_subkey(path)?;
            key.set_value(PROG_ID, &"")?;
        }
        Ok(())
    }

    /// Writes everything Windows needs to offer the app as a browser. It never
    /// makes it the default: only the user can, through `UserChoice`.
    pub fn register(&self, exe: &str) -> Result<(), RegistrationError> {
        if is_build_tree(exe) {
            return Err(RegistrationError::BuildTree);
        }
        self.write_prog_id(exe)?;
        self.write_start_menu(exe)?;
        self.write_capabilities(exe)?;
        self.write_registered_applications()?;
        self.write_open_with()
    }

    /// Removal has to mirror the write exactly, or the shell keeps offering an
    /// application that is no longer installed.
    pub fn unregister(&self) -> Result<(), RegistrationError> {
        let hkcu = Self::hkcu();
        let _ = hkcu.delete_subkey_all(format!(r"{}\{PROG_ID}", self.classes()));
        let _ = hkcu.delete_subkey_all(format!(
            r"{}\Clients\StartMenuInternet\{APP_NAME}",
            self.root
        ));
        let _ = hkcu.delete_subkey_all(format!(r"{}\{APP_NAME}", self.root));

        if let Ok(key) = hkcu.open_subkey_with_flags(
            format!(r"{}\RegisteredApplications", self.root),
            KEY_READ | KEY_WRITE,
        ) {
            let _ = key.delete_value(APP_NAME);
        }

        for ext in FILE_EXTENSIONS {
            let path = format!(r"{}\{ext}\OpenWithProgIds", self.classes());
            if let Ok(key) = hkcu.open_subkey_with_flags(&path, KEY_READ | KEY_WRITE) {
                let _ = key.delete_value(PROG_ID);
            }
        }
        Ok(())
    }

    /// Whether the app is offered at all. Whether the command points at the
    /// right binary is a separate question, and a separate answer.
    #[must_use]
    pub fn is_registered(&self) -> bool {
        self.registered_command().is_some()
    }

    /// The command as the shell will read it, so a caller can say what it points
    /// at rather than only whether it is ours.
    #[must_use]
    pub fn registered_command(&self) -> Option<String> {
        Self::hkcu()
            .open_subkey(format!(r"{}\{PROG_ID}\shell\open\command", self.classes()))
            .and_then(|k| k.get_value::<String, _>(""))
            .ok()
    }

    pub fn is_registered_as(&self, exe: &str) -> bool {
        Self::hkcu()
            .open_subkey(format!(r"{}\{PROG_ID}\shell\open\command", self.classes()))
            .and_then(|k| k.get_value::<String, _>(""))
            .is_ok_and(|command| {
                command
                    .to_ascii_lowercase()
                    .starts_with(&quoted(exe).to_ascii_lowercase())
            })
    }
}

/// The taskbar paints itself light or dark independently of the app theme, and
/// it never recolours a tray icon: the right one has to be handed over.
#[must_use]
pub fn taskbar_is_light() -> bool {
    RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey(r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize")
        .and_then(|k| k.get_value::<u32, _>("SystemUsesLightTheme"))
        .is_ok_and(|v| v == 1)
}

/// A separate setting from the taskbar one in Windows 11: an app may be dark on
/// a light taskbar, and the picker has to follow this one.
#[must_use]
pub fn windows_are_light() -> bool {
    RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey(r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize")
        .and_then(|k| k.get_value::<u32, _>("AppsUseLightTheme"))
        .is_ok_and(|v| v == 1)
}

/// Matches exactly, never by substring: a third-party ProgId embedding our name
/// would otherwise read as ours.
#[must_use]
pub fn prog_id_is_ours(prog_id: &str) -> bool {
    prog_id.eq_ignore_ascii_case(PROG_ID)
}

/// What the user actually chose. Windows owns these keys and no application may
/// write them, so this is a read and nothing else.
#[must_use]
pub fn is_default_browser() -> bool {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    USER_CHOICE_PATHS.iter().take(2).all(|path| {
        hkcu.open_subkey(path)
            .and_then(|k| k.get_value::<String, _>("ProgId"))
            .is_ok_and(|id| prog_id_is_ours(&id))
    })
}

/// Which associations the app holds and which another application took, so the
/// interface can say what is wrong instead of just that something is.
#[must_use]
pub fn association_report() -> Vec<(String, bool)> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    USER_CHOICE_PATHS
        .iter()
        .map(|path| {
            let held = hkcu
                .open_subkey(path)
                .and_then(|k| k.get_value::<String, _>("ProgId"))
                .is_ok_and(|id| prog_id_is_ours(&id));
            let name = path.rsplit('\\').nth(1).unwrap_or(path).to_owned();
            (name, held)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One root per test: they run in parallel and a shared root means each
    /// one's cleanup deletes the others' keys mid-assertion.
    fn scratch(name: &str) -> String {
        format!(r"Software\LinkUnbound-test\{name}")
    }

    fn scrub(root: &str) {
        let _ = RegKey::predef(HKEY_CURRENT_USER).delete_subkey_all(root);
    }

    #[test]
    fn registering_writes_the_keys_the_shell_looks_for() {
        let root = scratch("writes");
        let test_root = root.as_str();
        scrub(test_root);
        let reg = Registration::under(test_root);
        reg.register(r"C:\Apps\linkunbound.exe").unwrap();

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let command: String = hkcu
            .open_subkey(format!(r"{test_root}\Classes\{PROG_ID}\shell\open\command"))
            .unwrap()
            .get_value("")
            .unwrap();
        assert_eq!(command, "\"C:\\Apps\\linkunbound.exe\" \"%1\"");

        let caps: String = hkcu
            .open_subkey(format!(r"{test_root}\RegisteredApplications"))
            .unwrap()
            .get_value(APP_NAME)
            .unwrap();
        assert_eq!(caps, format!(r"{test_root}\{APP_NAME}\Capabilities"));

        let https: String = hkcu
            .open_subkey(format!(
                r"{test_root}\{APP_NAME}\Capabilities\URLAssociations"
            ))
            .unwrap()
            .get_value("https")
            .unwrap();
        assert_eq!(https, PROG_ID);

        assert!(reg.is_registered_as(r"C:\Apps\linkunbound.exe"));
        scrub(test_root);
    }

    #[test]
    fn a_forward_slash_path_is_written_the_way_the_shell_reads_it() {
        let root = scratch("slashes");
        let test_root = root.as_str();
        scrub(test_root);
        let reg = Registration::under(test_root);
        reg.register("C:/Apps/linkunbound.exe").unwrap();
        let command: String = RegKey::predef(HKEY_CURRENT_USER)
            .open_subkey(format!(r"{test_root}\Classes\{PROG_ID}\shell\open\command"))
            .unwrap()
            .get_value("")
            .unwrap();
        assert!(command.starts_with("\"C:\\Apps\\linkunbound.exe\""));
        scrub(test_root);
    }

    #[test]
    fn unregistering_leaves_nothing_behind() {
        let root = scratch("removal");
        let test_root = root.as_str();
        scrub(test_root);
        let reg = Registration::under(test_root);
        reg.register(r"C:\Apps\linkunbound.exe").unwrap();
        reg.unregister().unwrap();

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        assert!(!reg.is_registered());
        assert!(
            hkcu.open_subkey(format!(r"{test_root}\Classes\{PROG_ID}"))
                .is_err()
        );
        assert!(
            hkcu.open_subkey(format!(r"{test_root}\Clients\StartMenuInternet\{APP_NAME}"))
                .is_err()
        );

        for ext in FILE_EXTENSIONS {
            if let Ok(key) = hkcu.open_subkey(format!(r"{test_root}\Classes\{ext}\OpenWithProgIds"))
            {
                assert!(key.get_value::<String, _>(PROG_ID).is_err());
            }
        }
        scrub(test_root);
    }

    #[test]
    fn a_build_tree_is_refused_and_writes_nothing() {
        let root = scratch("buildtree");
        let test_root = root.as_str();
        scrub(test_root);
        let reg = Registration::under(test_root);

        let err = reg
            .register(r"D:\Code\LinkUnbound\target\release\linkunbound.exe")
            .unwrap_err();
        assert!(matches!(err, RegistrationError::BuildTree));
        assert!(!reg.is_registered());
        assert!(
            RegKey::predef(HKEY_CURRENT_USER)
                .open_subkey(format!(r"{test_root}\Classes\{PROG_ID}"))
                .is_err()
        );
        scrub(test_root);
    }

    #[test]
    fn an_installed_path_is_not_mistaken_for_a_build_tree() {
        assert!(!is_build_tree(
            r"C:\Program Files\LinkUnbound\linkunbound.exe"
        ));
        assert!(!is_build_tree(
            r"C:\Users\x\AppData\Local\LinkUnbound\linkunbound.exe"
        ));
        assert!(is_build_tree(r"D:\repo\target\debug\linkunbound.exe"));
        assert!(is_build_tree("D:/repo/target/release/linkunbound.exe"));
    }

    #[test]
    fn an_installed_path_that_merely_contains_the_word_target_is_not_a_build_tree() {
        assert!(!is_build_tree(
            r"C:\Program Files\LinkUnbound Target\linkunbound.exe"
        ));
    }

    #[test]
    fn a_target_triple_build_is_still_recognised_as_a_build_tree() {
        assert!(is_build_tree(
            r"D:\repo\target\x86_64-pc-windows-msvc\release\linkunbound.exe"
        ));
        assert!(is_build_tree(
            r"D:\repo\target\x86_64-pc-windows-msvc\debug\linkunbound.exe"
        ));
    }

    #[test]
    fn a_custom_cargo_profile_is_still_recognised_as_a_build_tree() {
        assert!(is_build_tree(r"D:\repo\target\dist\linkunbound.exe"));
    }

    #[test]
    fn a_different_executable_whose_name_merely_contains_ours_is_not_registered_as_us() {
        let root = scratch("impostor");
        let test_root = root.as_str();
        scrub(test_root);
        let reg = Registration::under(test_root);

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let (command, _) = hkcu
            .create_subkey(format!(r"{test_root}\Classes\{PROG_ID}\shell\open\command"))
            .unwrap();
        command
            .set_value("", &"\"C:\\Temp\\linkunbound-helper.exe\" \"%1\"")
            .unwrap();

        assert!(
            reg.is_registered(),
            "something is registered, whoever it belongs to"
        );
        assert!(
            !reg.is_registered_as(r"C:\Temp\linkunbound.exe"),
            "a name that merely contains ours is not ours"
        );
        scrub(test_root);
    }

    #[test]
    fn a_third_party_prog_id_embedding_our_name_is_not_ours() {
        assert!(prog_id_is_ours("LinkUnboundURL"));
        assert!(prog_id_is_ours("linkunboundurl"));
        assert!(!prog_id_is_ours("NotLinkUnboundURL"));
        assert!(!prog_id_is_ours("LinkUnboundURLPro"));
    }
}
