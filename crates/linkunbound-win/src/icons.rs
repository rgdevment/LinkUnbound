use std::fs;
use std::path::{Path, PathBuf};

use crate::native::file_icon;

/// Identifies the entry by the executable rather than by the browser id, which
/// is recycled: deleting `custom-2` and adding another browser handed the
/// newcomer the dead one's icon, because its exe is older than the leftover PNG.
fn fingerprint(exe: &str) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in exe.to_ascii_lowercase().bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x1000_0000_01b3);
    }
    hash
}

/// Extraction touches the shell and the GDI, so a click must not pay for it
/// twice. The side rides in the name: asking for a different one has to miss the
/// cache, or the picker would scale yesterday's size and lose its edges. And a
/// browser that updated gets read again, or its icon stays wrong for good.
#[must_use]
pub fn cached_or_extract(exe: &str, id: &str, dir: &Path, side: u32) -> Option<PathBuf> {
    // The id comes from a file the user can edit, and `Path::join` lets an
    // absolute or UNC one throw the directory away entirely.
    let named = linkunbound_core::as_file_name(id);
    let target = dir.join(format!("{named}-{:016x}-{side}.png", fingerprint(exe)));
    let source = fs::metadata(exe).and_then(|m| m.modified()).ok();
    match (fs::metadata(&target).and_then(|m| m.modified()), source) {
        (Ok(cached), Some(built)) if cached >= built => return Some(target),
        (Ok(_), None) => return Some(target),
        _ => {}
    }

    let (width, height, pixels) = file_icon(Path::new(exe), i32::try_from(side).ok()?)?;
    fs::create_dir_all(dir).ok()?;

    // Named after this process, and swept on failure: a shared staging name let
    // two extractions of the same browser leave a half-written PNG behind, and
    // the mtime check would serve it for good.
    let staging = target.with_extension(format!("{}.tmp", std::process::id()));
    let written = (|| {
        let file = fs::File::create(&staging).ok()?;
        let mut encoder = png::Encoder::new(std::io::BufWriter::new(file), width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().ok()?;
        writer.write_image_data(&pixels).ok()?;
        Some(())
    })();
    if written.is_none() || fs::rename(&staging, &target).is_err() {
        let _ = fs::remove_file(&staging);
        return None;
    }
    Some(target)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Per process: the path was machine-global, so a second `cargo test` wiped
    /// this one's directory mid-run and the failures looked like product bugs.
    fn scratch(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("linkunbound-icons-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    /// Ids are handed back out when a browser is deleted. Keyed on the id
    /// alone, the newcomer hit the dead one's PNG and wore its icon for good,
    /// because a freshly installed exe is older than yesterday's cache file.
    #[test]
    fn two_browsers_sharing_an_id_do_not_share_an_icon() {
        let dir = scratch("recycled");
        let first = r"C:\Windows\explorer.exe";
        let second = r"C:\Windows\System32\notepad.exe";
        if !Path::new(first).exists() || !Path::new(second).exists() {
            return;
        }
        let one = cached_or_extract(first, "custom-2", &dir, 24).expect("explorer has an icon");
        let two = cached_or_extract(second, "custom-2", &dir, 24).expect("notepad has an icon");
        assert_ne!(one, two, "the same id must not mean the same cache file");
    }

    /// The id reaches this from a file the user can edit, and `Path::join`
    /// discards the directory outright when handed an absolute or UNC one.
    #[test]
    fn an_id_cannot_send_the_cache_file_somewhere_else() {
        let dir = scratch("escape");
        let explorer = r"C:\Windows\explorer.exe";
        if !Path::new(explorer).exists() {
            return;
        }
        for hostile in [r"..\..\escaped", r"\\attacker.test\share\x", r"C:\evil"] {
            let written = cached_or_extract(explorer, hostile, &dir, 24).expect("should cache");
            assert!(
                written.starts_with(&dir),
                "{hostile} escaped the cache directory: {}",
                written.display()
            );
        }
    }

    #[test]
    fn an_executable_that_is_not_there_yields_nothing() {
        let dir = scratch("missing");
        assert!(cached_or_extract(r"C:\nope\ghost.exe", "ghost", &dir, 24).is_none());
    }

    #[test]
    fn a_real_executable_becomes_a_png_that_is_reused() {
        let dir = scratch("real");
        let explorer = r"C:\Windows\explorer.exe";
        if !Path::new(explorer).exists() {
            return;
        }

        let first = cached_or_extract(explorer, "explorer", &dir, 24).expect("should extract");
        assert!(first.exists());
        assert_eq!(first.extension().unwrap(), "png");

        let bytes = fs::read(&first).unwrap();
        assert_eq!(&bytes[1..4], b"PNG");
        assert!(bytes.len() > 200);

        let again =
            cached_or_extract(explorer, "explorer", &dir, 24).expect("should hit the cache");
        assert_eq!(first, again);

        let leftovers: Vec<_> = fs::read_dir(&dir)
            .unwrap()
            .flatten()
            .filter(|e| e.path().extension().is_some_and(|x| x == "tmp"))
            .collect();
        assert!(leftovers.is_empty());

        let _ = fs::remove_dir_all(&dir);
    }
}

#[cfg(test)]
mod probe {
    use super::cached_or_extract;
    use crate::installed_browsers;

    /// `cargo test -p linkunbound-win -- --ignored --nocapture`
    #[test]
    #[ignore = "extracts from the local machine rather than asserting"]
    fn extract_every_installed_browser() {
        let dir = std::env::temp_dir().join("linkunbound-icons-probe");
        let _ = std::fs::remove_dir_all(&dir);
        for browser in installed_browsers() {
            match cached_or_extract(&browser.exe, &browser.id, &dir, 24) {
                Some(path) => {
                    let bytes = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                    println!("{:<34} {} bytes", browser.name, bytes);
                }
                None => println!("{:<34} SIN ICONO", browser.name),
            }
        }
        println!("en {}", dir.display());
    }
}
