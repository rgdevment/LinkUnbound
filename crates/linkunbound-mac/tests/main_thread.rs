//! Runs without a harness so that it owns the main thread, which is the one
//! thread AppKit lets a window be made on and the one `listen` needs.
#![cfg_attr(target_os = "macos", allow(unsafe_code))]

/// The listing protocol nextest speaks to a harness of its own: one test, never ignored.
fn listed_instead_of_run() -> bool {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--list") {
        if !args.iter().any(|a| a == "--ignored") {
            println!("main_thread: test");
        }
        return true;
    }
    false
}

#[cfg(not(target_os = "macos"))]
fn main() {
    let _ = listed_instead_of_run();
}

#[cfg(target_os = "macos")]
fn main() {
    if listed_instead_of_run() {
        return;
    }
    use std::cell::RefCell;
    use std::ptr::NonNull;
    use std::rc::Rc;

    use linkunbound_mac::{
        Event, cursor, dress_window, is_in_front, keep_off_the_taskbar,
        let_whoever_opens_next_come_forward, listen, work_area_at,
    };
    use objc2::MainThreadOnly;
    use objc2::rc::Retained;
    use objc2_app_kit::{
        NSAppearanceCustomization, NSApplication, NSApplicationActivationPolicy,
        NSBackingStoreType, NSFloatingWindowLevel, NSWindow, NSWindowCollectionBehavior,
        NSWindowStyleMask,
    };
    use objc2_core_services::keyDirectObject;
    use objc2_foundation::{
        MainThreadMarker, NSAppleEventDescriptor, NSAppleEventManager, NSPoint, NSRect, NSSize,
        NSString,
    };

    let mtm = MainThreadMarker::new().expect("a harness-less test owns the main thread");
    let app = NSApplication::sharedApplication(mtm);

    let (x, y, width, height) = work_area_at(0, 0).expect("the primary screen");
    assert!(width > 0 && height > 0, "{width}x{height}");
    assert!(
        y >= 0,
        "the menu bar pushes the work area down, never up: {y}"
    );
    assert_eq!(
        work_area_at(i32::MAX / 2, i32::MAX / 2),
        Some((x, y, width, height)),
        "a point on no screen is placed on the first one"
    );
    let (cx, cy) = cursor().expect("a pointer somewhere");
    assert!(cx > i32::MIN && cy > i32::MIN);

    // SAFETY: made and used on the main thread, never shown, dropped at the end.
    let window = unsafe {
        NSWindow::initWithContentRect_styleMask_backing_defer(
            NSWindow::alloc(mtm),
            NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(10.0, 10.0)),
            NSWindowStyleMask::Borderless,
            NSBackingStoreType::Buffered,
            false,
        )
    };
    let view = window.contentView().expect("a content view");
    let handle = Retained::as_ptr(&view) as isize;
    keep_off_the_taskbar(handle, 10.0);
    assert_eq!(window.level(), NSFloatingWindowLevel);
    let behaviour = window.collectionBehavior();
    assert!(behaviour.contains(NSWindowCollectionBehavior::CanJoinAllSpaces));
    assert!(behaviour.contains(NSWindowCollectionBehavior::FullScreenAuxiliary));
    assert!(!window.isOpaque(), "the corners are drawn by the layer");
    assert!(window.hasShadow());
    let layer = view.layer().expect("a layer to round");
    assert_eq!(layer.cornerRadius(), 10.0);
    assert!(layer.masksToBounds());

    let window_handle = Retained::as_ptr(&window) as isize;
    let named = |window: &NSWindow| window.appearance().map(|a| a.name().to_string());
    dress_window(window_handle, Some(true));
    assert_eq!(named(&window).as_deref(), Some("NSAppearanceNameDarkAqua"));
    dress_window(window_handle, Some(false));
    assert_eq!(named(&window).as_deref(), Some("NSAppearanceNameAqua"));
    dress_window(window_handle, None);
    assert_eq!(named(&window), None, "following the system again");

    assert!(!is_in_front(handle), "never ordered front");
    let_whoever_opens_next_come_forward();
    assert_eq!(
        app.activationPolicy(),
        NSApplicationActivationPolicy::Accessory
    );

    let seen: Rc<RefCell<Vec<Event>>> = Rc::new(RefCell::new(Vec::new()));
    let listening = {
        let seen = Rc::clone(&seen);
        listen(move |event| seen.borrow_mut().push(event)).expect("a listener on the main thread")
    };
    let link = NSAppleEventDescriptor::appleEventWithEventClass_eventID_targetDescriptor_returnID_transactionID(
        0x4755_524C, 0x4755_524C, None, -1, 0,
    );
    link.setParamDescriptor_forKeyword(
        &NSAppleEventDescriptor::descriptorWithString(&NSString::from_str("https://example.com/x")),
        keyDirectObject,
    );
    let reply = NSAppleEventDescriptor::nullDescriptor();
    let deliver = || {
        // SAFETY: both descriptors outlive the call; the handler takes no ref con.
        unsafe {
            NSAppleEventManager::sharedAppleEventManager()
                .dispatchRawAppleEvent_withRawReply_handlerRefCon(
                    NonNull::new(link.aeDesc().cast_mut()).expect("an event"),
                    NonNull::new(reply.aeDesc().cast_mut()).expect("a reply"),
                    std::ptr::null_mut(),
                )
        }
    };
    assert_eq!(deliver(), 0, "the handler took the event");
    assert_eq!(
        *seen.borrow(),
        vec![Event::Link("https://example.com/x".to_owned())]
    );
    drop(listening);
    assert_ne!(deliver(), 0, "the handler leaves with the listener");
    assert_eq!(seen.borrow().len(), 1);

    println!("main thread: window, screen and events behave");
}
