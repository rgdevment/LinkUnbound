use linkunbound_core::{Origin, Store, data_dir};

/// Noted before the rules are asked, so an app a rule always answers is listed as well.
#[must_use]
pub fn noted(source: Option<Origin>) -> Option<Origin> {
    if let Some(origin) = &source {
        Store::at(data_dir()).note_app_seen(origin, program_of(origin).as_deref());
    }
    source
}

#[cfg(windows)]
fn program_of(origin: &Origin) -> Option<String> {
    let front = linkunbound_win::foreground_process_path()?;
    let stem = std::path::Path::new(&front)
        .file_stem()?
        .to_str()?
        .to_lowercase();
    (stem == origin.key).then_some(front)
}

#[cfg(not(windows))]
fn program_of(_origin: &Origin) -> Option<String> {
    None
}
