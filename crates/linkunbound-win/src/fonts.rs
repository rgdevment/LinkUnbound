//! DirectWrite has no safe wrapper; `scripts/rules.sh` and the audit in `lib.rs` name this file.
#![allow(unsafe_code)]

use linkunbound_core::fonts::{Family, substituted};
use windows::Win32::Globalization::GetUserDefaultLocaleName;
use windows::Win32::Graphics::DirectWrite::{
    DWRITE_FACTORY_TYPE_SHARED, DWRITE_FONT_STRETCH_NORMAL, DWRITE_FONT_STYLE_NORMAL,
    DWRITE_FONT_WEIGHT_REGULAR, DWriteCreateFactory, IDWriteFactory, IDWriteFontCollection,
    IDWriteFontFamily, IDWriteLocalizedStrings,
};
use windows::Win32::Graphics::Gdi::LOGFONTW;
use windows::Win32::UI::WindowsAndMessaging::{
    NONCLIENTMETRICSW, SPI_GETNONCLIENTMETRICS, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS,
    SystemParametersInfoW,
};
use windows::core::{BOOL, HSTRING};
use winreg::RegKey;
use winreg::enums::HKEY_LOCAL_MACHINE;

const SUBSTITUTES: &str = r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\FontSubstitutes";
const ENGLISH: &str = "en-us";

#[must_use]
pub fn families() -> Vec<Family> {
    let Some(collection) = system_collection(true) else {
        return Vec::new();
    };
    let user = user_locale();
    let count = unsafe { collection.GetFontFamilyCount() };
    (0..count)
        .filter_map(|index| unsafe { collection.GetFontFamily(index) }.ok())
        .filter_map(|family| family_of(&family, user.as_deref()))
        .collect()
}

/// Looking for new fonts costs tens of milliseconds on the way to the picker, so only a miss does.
#[must_use]
pub fn installed(name: &str) -> bool {
    knows(name, false) || knows(name, true)
}

fn knows(name: &str, new_fonts: bool) -> bool {
    let Some(collection) = system_collection(new_fonts) else {
        return false;
    };
    let (mut index, mut exists) = (0u32, BOOL(0));
    let name = HSTRING::from(name);
    unsafe { collection.FindFamilyName(&name, &raw mut index, &raw mut exists) }.is_ok()
        && exists.as_bool()
}

#[must_use]
pub fn system() -> Option<String> {
    let face = message_face()?;
    family_of_gdi(&substituted(&face, &substitutes()))
}

fn message_face() -> Option<String> {
    let mut metrics = NONCLIENTMETRICSW {
        cbSize: u32::try_from(std::mem::size_of::<NONCLIENTMETRICSW>()).ok()?,
        ..Default::default()
    };
    unsafe {
        SystemParametersInfoW(
            SPI_GETNONCLIENTMETRICS,
            metrics.cbSize,
            Some((&raw mut metrics).cast()),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        )
    }
    .ok()?;
    let face = wide_until_nul(&metrics.lfMessageFont.lfFaceName);
    (!face.is_empty()).then_some(face)
}

fn substitutes() -> Vec<(String, String)> {
    let Ok(key) = RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey(SUBSTITUTES) else {
        return Vec::new();
    };
    key.enum_values()
        .filter_map(Result::ok)
        .map(|(name, value)| (name, value.to_string()))
        .collect()
}

/// A GDI face may name a weight, as in `Segoe UI Semibold`; the renderer wants the family.
fn family_of_gdi(face: &str) -> Option<String> {
    let mut logfont = LOGFONTW::default();
    let wide: Vec<u16> = face.encode_utf16().collect();
    if wide.is_empty() || wide.len() >= logfont.lfFaceName.len() {
        return None;
    }
    logfont.lfFaceName[..wide.len()].copy_from_slice(&wide);
    let interop = unsafe { factory()?.GetGdiInterop() }.ok()?;
    let font = unsafe { interop.CreateFontFromLOGFONT(&raw const logfont) }.ok()?;
    let names = unsafe {
        font.GetFontFamily()
            .and_then(|family| family.GetFamilyNames())
    }
    .ok()?;
    name_in(&names, ENGLISH).or_else(|| name_at(&names, 0))
}

fn factory() -> Option<IDWriteFactory> {
    unsafe { DWriteCreateFactory::<IDWriteFactory>(DWRITE_FACTORY_TYPE_SHARED) }.ok()
}

fn system_collection(new_fonts: bool) -> Option<IDWriteFontCollection> {
    let factory = factory()?;
    let mut collection = None;
    unsafe { factory.GetSystemFontCollection(&raw mut collection, new_fonts) }.ok()?;
    collection
}

fn family_of(family: &IDWriteFontFamily, user: Option<&str>) -> Option<Family> {
    let names = unsafe { family.GetFamilyNames() }.ok()?;
    let name = name_in(&names, ENGLISH).or_else(|| name_at(&names, 0))?;
    let local = user
        .and_then(|locale| name_in(&names, locale))
        .unwrap_or_else(|| name.clone());
    let font = unsafe {
        family.GetFirstMatchingFont(
            DWRITE_FONT_WEIGHT_REGULAR,
            DWRITE_FONT_STRETCH_NORMAL,
            DWRITE_FONT_STYLE_NORMAL,
        )
    }
    .ok()?;
    Some(Family {
        name,
        local,
        symbol: unsafe { font.IsSymbolFont() }.as_bool(),
    })
}

fn name_in(names: &IDWriteLocalizedStrings, locale: &str) -> Option<String> {
    let (mut index, mut exists) = (0u32, BOOL(0));
    let locale = HSTRING::from(locale);
    unsafe { names.FindLocaleName(&locale, &raw mut index, &raw mut exists) }.ok()?;
    if !exists.as_bool() {
        return None;
    }
    name_at(names, index)
}

fn name_at(names: &IDWriteLocalizedStrings, index: u32) -> Option<String> {
    let length = unsafe { names.GetStringLength(index) }.ok()?;
    let mut buffer = vec![0u16; length as usize + 1];
    unsafe { names.GetString(index, &mut buffer) }.ok()?;
    let said = wide_until_nul(&buffer);
    (!said.is_empty()).then_some(said)
}

fn user_locale() -> Option<String> {
    let mut buffer = [0u16; 85];
    let written = unsafe { GetUserDefaultLocaleName(&mut buffer) };
    (written > 0).then(|| wide_until_nul(&buffer))
}

fn wide_until_nul(wide: &[u16]) -> String {
    let end = wide.iter().position(|&one| one == 0).unwrap_or(wide.len());
    String::from_utf16_lossy(&wide[..end])
}

#[cfg(test)]
mod tests {
    use super::{families, family_of_gdi, installed, message_face, substitutes, system};

    #[test]
    fn a_plain_windows_lists_segoe_ui_and_knows_marlett_draws_pictures() {
        let found = families();
        let named = |name: &str| found.iter().find(|one| one.name == name);
        assert!(
            named("Segoe UI").is_some_and(|one| !one.symbol),
            "{found:?}"
        );
        assert!(named("Marlett").is_some_and(|one| one.symbol), "{found:?}");
        assert!(
            found
                .iter()
                .all(|one| !one.name.is_empty() && !one.local.is_empty())
        );
    }

    #[test]
    fn a_family_is_installed_only_when_directwrite_knows_it() {
        assert!(installed("Segoe UI"));
        assert!(installed("segoe ui"));
        assert!(!installed("Zzz Nobody Installed This"));
        assert!(!installed(""));
    }

    #[test]
    fn the_system_family_comes_from_the_message_font() {
        assert!(message_face().is_some_and(|face| !face.is_empty()));
        assert!(system().is_some_and(|family| installed(&family)));
    }

    #[test]
    fn a_gdi_face_maps_to_its_directwrite_family_and_an_unknown_one_to_nothing() {
        assert_eq!(family_of_gdi("Segoe UI").as_deref(), Some("Segoe UI"));
        assert_eq!(
            family_of_gdi("Segoe UI Semibold").as_deref(),
            Some("Segoe UI")
        );
        assert_eq!(family_of_gdi("Zzz Nobody Installed This"), None);
        assert_eq!(family_of_gdi(&"x".repeat(64)), None);
        assert_eq!(family_of_gdi(""), None);
    }

    #[test]
    fn the_substitutes_name_the_shell_dialog_font() {
        let found = substitutes();
        assert!(
            found
                .iter()
                .any(|(key, _)| key.eq_ignore_ascii_case("MS Shell Dlg")),
            "{found:?}"
        );
    }
}
