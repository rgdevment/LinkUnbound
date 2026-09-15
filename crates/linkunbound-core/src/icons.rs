use std::fs;
use std::path::{Path, PathBuf};

/// Identifies the entry by the source rather than by the browser id, which is recycled:
/// deleting `custom-2` and adding another browser would hand the newcomer the dead one's icon.
fn fingerprint(source: &str) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in source.to_ascii_lowercase().bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x1000_0000_01b3);
    }
    hash
}

/// The side rides in the name so another size misses the cache, and a source that changed
/// since the picture was made is read again.
pub fn cached_icon(
    source: &str,
    id: &str,
    dir: &Path,
    side: u32,
    draw: impl FnOnce(&Path, u32) -> Option<Vec<u8>>,
) -> Option<PathBuf> {
    let named = crate::as_file_name(id);
    let target = dir.join(format!("{named}-{:016x}-{side}.png", fingerprint(source)));
    let built = fs::metadata(source).and_then(|m| m.modified()).ok();
    match (fs::metadata(&target).and_then(|m| m.modified()), built) {
        (Ok(cached), Some(built)) if cached >= built => return Some(target),
        (Ok(_), None) => return Some(target),
        _ => {}
    }

    let png = draw(Path::new(source), side)?;
    fs::create_dir_all(dir).ok()?;
    let staging = target.with_extension(format!("{}.tmp", std::process::id()));
    if fs::write(&staging, png).is_err() || fs::rename(&staging, &target).is_err() {
        let _ = fs::remove_file(&staging);
        return None;
    }
    Some(target)
}

#[cfg(test)]
mod tests {
    use super::cached_icon;
    use std::path::PathBuf;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("linkunbound-icons-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("a scratch directory");
        dir.join(name)
    }

    #[test]
    fn a_picture_is_drawn_once_and_served_from_then_on() {
        let source = scratch("browser.bin");
        std::fs::write(&source, b"x").expect("a source");
        let dir = scratch("icons");
        let mut drawn = 0;
        let mut draw = |_: &std::path::Path, _: u32| {
            drawn += 1;
            Some(b"png".to_vec())
        };

        let first = cached_icon(&source.to_string_lossy(), "chrome", &dir, 24, &mut draw)
            .expect("a picture");
        let again = cached_icon(&source.to_string_lossy(), "chrome", &dir, 24, &mut draw)
            .expect("the same picture");
        assert_eq!(first, again);
        assert_eq!(drawn, 1);
        assert_eq!(std::fs::read(&first).expect("readable"), b"png");
    }

    #[test]
    fn another_side_or_another_source_is_another_picture() {
        let one = scratch("one.bin");
        let two = scratch("two.bin");
        std::fs::write(&one, b"x").expect("a source");
        std::fs::write(&two, b"x").expect("a source");
        let dir = scratch("icons-apart");
        let draw = |_: &std::path::Path, side: u32| Some(side.to_le_bytes().to_vec());

        let small = cached_icon(&one.to_string_lossy(), "b", &dir, 16, draw).expect("16");
        let large = cached_icon(&one.to_string_lossy(), "b", &dir, 32, draw).expect("32");
        let other = cached_icon(&two.to_string_lossy(), "b", &dir, 16, draw).expect("other");
        assert_ne!(small, large);
        assert_ne!(small, other);
    }

    #[test]
    fn a_picture_that_cannot_be_drawn_leaves_nothing_behind() {
        let source = scratch("blank.bin");
        std::fs::write(&source, b"x").expect("a source");
        let dir = scratch("icons-none");
        assert!(cached_icon(&source.to_string_lossy(), "b", &dir, 24, |_, _| None).is_none());
        let left: Vec<_> = std::fs::read_dir(&dir)
            .map(|d| d.flatten().collect())
            .unwrap_or_default();
        assert!(left.is_empty(), "{left:?}");
    }

    #[test]
    fn an_id_that_names_a_path_stays_inside_the_directory() {
        let source = scratch("safe.bin");
        std::fs::write(&source, b"x").expect("a source");
        let dir = scratch("icons-safe");
        let made = cached_icon(&source.to_string_lossy(), "../../evil", &dir, 24, |_, _| {
            Some(b"png".to_vec())
        })
        .expect("a picture");
        assert!(made.starts_with(&dir), "{made:?}");
    }
}
