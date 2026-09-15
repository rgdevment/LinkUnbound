#![allow(unsafe_code)]

use std::path::{Path, PathBuf};

use objc2::AllocAnyThread;
use objc2_app_kit::{
    NSBitmapImageFileType, NSBitmapImageRep, NSCompositingOperation, NSDeviceRGBColorSpace,
    NSGraphicsContext, NSImageInterpolation, NSWorkspace,
};
use objc2_foundation::{NSDictionary, NSPoint, NSRect, NSSize, NSString};

fn drawn(path: &Path, side: u32) -> Option<Vec<u8>> {
    if !path.exists() {
        return None;
    }
    let pixels = isize::try_from(side).ok()?;
    let image =
        NSWorkspace::sharedWorkspace().iconForFile(&NSString::from_str(&path.to_string_lossy()));
    let canvas = unsafe {
        NSBitmapImageRep::initWithBitmapDataPlanes_pixelsWide_pixelsHigh_bitsPerSample_samplesPerPixel_hasAlpha_isPlanar_colorSpaceName_bytesPerRow_bitsPerPixel(
            NSBitmapImageRep::alloc(),
            std::ptr::null_mut(),
            pixels,
            pixels,
            8,
            4,
            true,
            false,
            NSDeviceRGBColorSpace,
            0,
            0,
        )
    }?;
    let size = NSSize::new(f64::from(side), f64::from(side));
    canvas.setSize(size);

    let context = NSGraphicsContext::graphicsContextWithBitmapImageRep(&canvas)?;
    NSGraphicsContext::saveGraphicsState_class();
    NSGraphicsContext::setCurrentContext(Some(&context));
    context.setImageInterpolation(NSImageInterpolation::High);
    image.drawInRect_fromRect_operation_fraction(
        NSRect::new(NSPoint::new(0.0, 0.0), size),
        NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(0.0, 0.0)),
        NSCompositingOperation::SourceOver,
        1.0,
    );
    NSGraphicsContext::restoreGraphicsState_class();

    let png = unsafe {
        canvas.representationUsingType_properties(NSBitmapImageFileType::PNG, &NSDictionary::new())
    }?;
    Some(png.to_vec())
}

#[must_use]
pub fn icon_for(app: &str, id: &str, dir: &Path, side: u32) -> Option<PathBuf> {
    linkunbound_core::cached_icon(app, id, dir, side, drawn)
}

#[cfg(test)]
mod tests {
    use super::{drawn, icon_for};
    use std::path::Path;

    fn png_size(bytes: &[u8]) -> (u32, u32) {
        let decoder = png::Decoder::new(std::io::Cursor::new(bytes));
        let reader = decoder.read_info().expect("a png");
        (reader.info().width, reader.info().height)
    }

    #[test]
    fn an_application_is_drawn_at_exactly_the_side_asked_for() {
        let bytes =
            drawn(Path::new("/System/Applications/Utilities/Terminal.app"), 24).expect("a picture");
        assert_eq!(png_size(&bytes), (24, 24));
        let larger =
            drawn(Path::new("/System/Applications/Utilities/Terminal.app"), 48).expect("a picture");
        assert_eq!(png_size(&larger), (48, 48));
    }

    #[test]
    fn a_path_that_names_nothing_gets_no_picture_rather_than_a_generic_one() {
        assert!(drawn(Path::new("/nowhere/Browser.app"), 24).is_none());
        assert!(drawn(Path::new("x.exe"), 24).is_none());
    }

    #[test]
    fn the_picture_lands_in_the_directory_named_after_the_browser() {
        let dir =
            std::env::temp_dir().join(format!("linkunbound-mac-icons-{}", std::process::id()));
        let made = icon_for(
            "/System/Applications/Utilities/Terminal.app",
            "terminal",
            &dir,
            24,
        )
        .expect("a picture");
        assert!(made.starts_with(&dir));
        assert!(
            made.file_name()
                .is_some_and(|n| n.to_string_lossy().starts_with("terminal-"))
        );
        assert_eq!(png_size(&std::fs::read(&made).expect("readable")), (24, 24));
    }
}
