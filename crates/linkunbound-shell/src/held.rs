//! Links that reach the socket before the event loop runs, kept until there is a window.

/// Links the socket took before the loop could: the copy that handed them over believes they
/// were delivered, so they are kept here until there is a window to deliver them to.
static HELD_FOR_THE_LOOP: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());

pub fn hold_for_the_loop(url: String) {
    if let Ok(mut held) = HELD_FOR_THE_LOOP.lock() {
        held.push(url);
    }
}

pub fn release_to_the_loop() {
    let held: Vec<String> = HELD_FOR_THE_LOOP
        .lock()
        .map(|mut held| held.drain(..).collect())
        .unwrap_or_default();
    for url in held {
        let _ = slint::invoke_from_event_loop(move || crate::arrived(url));
    }
}

pub fn handed_to_the_loop(url: String) {
    let raw = url.clone();
    if slint::invoke_from_event_loop(move || crate::arrived(url)).is_err() {
        hold_for_the_loop(raw);
    }
}

#[cfg(test)]
mod tests {
    use super::{HELD_FOR_THE_LOOP, hold_for_the_loop, release_to_the_loop};

    /// Held links leave the queue once handed on, so none is delivered twice.
    #[test]
    fn a_held_link_is_handed_on_once() {
        hold_for_the_loop("https://a.test/".to_owned());
        assert!(
            HELD_FOR_THE_LOOP
                .lock()
                .is_ok_and(|held| held.iter().any(|u| u == "https://a.test/"))
        );
        release_to_the_loop();
        assert!(
            HELD_FOR_THE_LOOP
                .lock()
                .is_ok_and(|held| !held.iter().any(|u| u == "https://a.test/"))
        );
    }
}
