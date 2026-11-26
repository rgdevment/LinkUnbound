//! CoreText has no safe wrapper; `scripts/rules.sh` and the audit in `lib.rs` name this file.
#![allow(unsafe_code)]

use linkunbound_core::fonts::Family;
use objc2_core_foundation::{CFArray, CFDictionary, CFRetained, CFString};
use objc2_core_text::{
    CTFont, CTFontDescriptor, CTFontManagerCopyAvailableFontFamilyNames, CTFontStylisticClass,
    CTFontSymbolicTraits, kCTFontFamilyNameAttribute, kCTFontFamilyNameKey,
};

#[must_use]
pub fn families() -> Vec<Family> {
    names().iter().map(|name| family_of(&name)).collect()
}

/// Whether CoreText lists a family by this name.
#[must_use]
pub fn installed(name: &str) -> bool {
    !name.is_empty()
        && names()
            .iter()
            .any(|one| one.to_string().eq_ignore_ascii_case(name))
}

fn names() -> CFRetained<CFArray<CFString>> {
    let names = unsafe { CTFontManagerCopyAvailableFontFamilyNames() };
    unsafe { CFRetained::cast_unchecked::<CFArray<CFString>>(names) }
}

fn family_of(name: &CFString) -> Family {
    let key = unsafe { kCTFontFamilyNameAttribute };
    let asked = CFDictionary::<CFString, CFString>::from_slices(&[key], &[name]);
    let descriptor = unsafe { CTFontDescriptor::with_attributes(asked.as_ref()) };
    let font = unsafe { CTFont::with_font_descriptor(&descriptor, 12.0, std::ptr::null()) };
    let traits = unsafe { font.symbolic_traits() };
    let class = traits.0 & CTFontSymbolicTraits::TraitClassMask.0;
    let family_key = unsafe { kCTFontFamilyNameKey };
    let local = unsafe { font.localized_name(family_key, std::ptr::null_mut()) };
    let name = name.to_string();
    Family {
        local: local.map_or_else(|| name.clone(), |local| local.to_string()),
        name,
        symbol: class == CTFontStylisticClass::ClassSymbolic.0,
    }
}

#[cfg(test)]
mod tests {
    use super::{families, installed};

    #[test]
    fn a_mac_lists_helvetica_and_knows_it_is_installed() {
        let found = families();
        assert!(
            found
                .iter()
                .any(|one| one.name == "Helvetica" && !one.symbol),
            "{found:?}"
        );
        assert!(installed("Helvetica"));
        assert!(installed("helvetica"));
        assert!(!installed("Zzz Nobody Installed This"));
        assert!(!installed(""));
    }
}
