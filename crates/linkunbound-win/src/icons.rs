use std::fs;
use std::path::{Path, PathBuf};

use crate::native::file_icon;

/// Extraction touches the shell and the GDI, so a click must not pay for it
/// twice. The side rides in the name: asking for a different one has to miss the
/// cache, or the picker would scale yesterday's size and lose its edges. And a
/// browser that updated gets read again, or its icon stays wrong for good.
#[must_use]
pub fn cached_or_extract(exe: &str, id: &str, dir: &Path, side: u32) -> Option<PathBuf> {
    let target = dir.join(format!("{id}-{side}.png"));
    let source = fs::metadata(exe).and_then(|m| m.modified()).ok();
    match (fs::metadata(&target).and_then(|m| m.modified()), source) {
        (Ok(cached), Some(built)) if cached >= built => return Some(target),
        (Ok(_), None) => return Some(target),
        _ => {}
    }

    let (width, height, pixels) = file_icon(Path::new(exe), i32::try_from(side).ok()?)?;
    fs::create_dir_all(dir).ok()?;

    let staging = target.with_extension("tmp");
    {
        let file = fs::File::create(&staging).ok()?;
        let mut encoder = png::Encoder::new(std::io::BufWriter::new(file), width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().ok()?;
        writer.write_image_data(&pixels).ok()?;
    }
    fs::rename(&staging, &target).ok()?;
    Some(target)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("linkunbound-icons-{name}"));
        let _ = fs::remove_dir_all(&dir);
        dir
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
