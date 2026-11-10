use linkunbound_core::{HUSHED, is_own_scheme};

/// A refused `linkunbound:` link came from a page or another process, never from a person starting the app.
#[must_use]
pub fn wants_no_window(args: &[String], incoming: Option<&str>) -> bool {
    args.iter().any(|a| a == HUSHED)
        || (incoming.is_none() && args.iter().skip(1).any(|a| is_own_scheme(a)))
}

pub fn comes_back_after_updates() {
    #[cfg(windows)]
    let _ = linkunbound_win::reopened_with(HUSHED);
}
