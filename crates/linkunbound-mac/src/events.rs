#![allow(unsafe_code)]

use std::path::PathBuf;
use std::ptr::NonNull;

use block2::RcBlock;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2::{DefinedClass, MainThreadOnly, define_class, msg_send, sel};
use objc2_app_kit::{
    NSApplicationDidFinishLaunchingNotification, NSApplicationWillFinishLaunchingNotification,
};
use objc2_core_services::{
    AEEventClass, AEEventID, kAEOpenApplication, kAEOpenDocuments, kAEReopenApplication,
    kCoreEventClass, keyAELaunchedAsLogInItem, keyAEPropData, keyDirectObject, typeFileURL,
};
use objc2_foundation::{
    MainThreadMarker, NSAppleEventDescriptor, NSAppleEventManager, NSNotification,
    NSNotificationCenter, NSObject, NSObjectProtocol,
};

const GET_URL: u32 = 0x4755_524C;
const CORE: AEEventClass = kCoreEventClass;
const OPEN_DOCUMENTS: AEEventID = kAEOpenDocuments;
const OPEN_APPLICATION: AEEventID = kAEOpenApplication;
const REOPEN_APPLICATION: AEEventID = kAEReopenApplication;

const HANDLED: [(AEEventClass, AEEventID); 4] = [
    (GET_URL, GET_URL),
    (CORE, OPEN_DOCUMENTS),
    (CORE, OPEN_APPLICATION),
    (CORE, REOPEN_APPLICATION),
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    Link(String),
    Document(PathBuf),
    Launched { as_login_item: bool },
    Reopened,
}

fn link_in(event: &NSAppleEventDescriptor) -> Option<String> {
    let text = event
        .paramDescriptorForKeyword(keyDirectObject)?
        .stringValue()?
        .to_string();
    (!text.is_empty()).then_some(text)
}

fn documents_in(event: &NSAppleEventDescriptor) -> Vec<PathBuf> {
    let Some(direct) = event.paramDescriptorForKeyword(keyDirectObject) else {
        return Vec::new();
    };
    let count = direct.numberOfItems();
    let items: Vec<Retained<NSAppleEventDescriptor>> = if count == 0 {
        vec![direct]
    } else {
        (1..=count)
            .filter_map(|i| direct.descriptorAtIndex(i))
            .collect()
    };
    items
        .iter()
        .filter_map(|item| {
            item.fileURLValue().or_else(|| {
                item.coerceToDescriptorType(typeFileURL)
                    .and_then(|coerced| coerced.fileURLValue())
            })
        })
        .filter_map(|url| url.path())
        .map(|path| PathBuf::from(path.to_string()))
        .collect()
}

fn as_login_item(event: &NSAppleEventDescriptor) -> bool {
    event
        .paramDescriptorForKeyword(keyAEPropData)
        .is_some_and(|prop| prop.enumCodeValue() == keyAELaunchedAsLogInItem)
}

fn read(event: &NSAppleEventDescriptor) -> Vec<Event> {
    match (event.eventClass(), event.eventID()) {
        (GET_URL, GET_URL) => link_in(event).map(Event::Link).into_iter().collect(),
        (CORE, OPEN_DOCUMENTS) => documents_in(event)
            .into_iter()
            .map(Event::Document)
            .collect(),
        (CORE, OPEN_APPLICATION) => vec![Event::Launched {
            as_login_item: as_login_item(event),
        }],
        (CORE, REOPEN_APPLICATION) => vec![Event::Reopened],
        _ => Vec::new(),
    }
}

type Identity = (AEEventClass, AEEventID, i16);

fn identity(event: &NSAppleEventDescriptor) -> Identity {
    (event.eventClass(), event.eventID(), event.returnID())
}

struct Ivars {
    on: Box<dyn Fn(Event)>,
    handled: std::cell::Cell<Option<Identity>>,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "LinkUnboundAppleEvents"]
    #[ivars = Ivars]
    struct Listener;

    impl Listener {
        #[unsafe(method(handleEvent:withReplyEvent:))]
        fn handle(&self, event: &NSAppleEventDescriptor, _reply: &NSAppleEventDescriptor) {
            self.deliver(event);
        }
    }

    unsafe impl NSObjectProtocol for Listener {}
);

impl Listener {
    fn new(mtm: MainThreadMarker, on: Box<dyn Fn(Event)>) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(Ivars {
            on,
            handled: std::cell::Cell::new(None),
        });
        unsafe { msg_send![super(this), init] }
    }

    fn deliver(&self, event: &NSAppleEventDescriptor) {
        self.ivars().handled.set(Some(identity(event)));
        for found in read(event) {
            (self.ivars().on)(found);
        }
    }

    /// The event that launched the process is handled inside `finishLaunching`
    /// by AppKit's own `oapp` and `odoc` handlers, which replace these until
    /// they are put back once launching is done: a login item would look like
    /// a launch by hand, and a double-clicked page would open nothing. It is
    /// still the current event then, so it is read from there, unless it was
    /// a link, which AppKit leaves to whoever claimed it and so arrived already.
    fn catch_up(&self) {
        let Some(current) = NSAppleEventManager::sharedAppleEventManager().currentAppleEvent()
        else {
            return;
        };
        if self.ivars().handled.get() != Some(identity(&current)) {
            self.deliver(&current);
        }
    }

    fn install(&self) {
        let manager = NSAppleEventManager::sharedAppleEventManager();
        for (class, id) in HANDLED {
            unsafe {
                manager.setEventHandler_andSelector_forEventClass_andEventID(
                    self,
                    sel!(handleEvent:withReplyEvent:),
                    class,
                    id,
                );
            }
        }
    }
}

pub struct Listening {
    _listener: Retained<Listener>,
    _observers: Vec<Retained<ProtocolObject<dyn NSObjectProtocol>>>,
}

impl Drop for Listening {
    fn drop(&mut self) {
        let manager = NSAppleEventManager::sharedAppleEventManager();
        for (class, id) in HANDLED {
            manager.removeEventHandlerForEventClass_andEventID(class, id);
        }
        for observer in &self._observers {
            unsafe {
                NSNotificationCenter::defaultCenter().removeObserver(observer.as_ref());
            }
        }
    }
}

#[must_use]
fn on_launching(
    name: &'static objc2_foundation::NSNotificationName,
    listener: &Retained<Listener>,
    catch_up: bool,
) -> Retained<ProtocolObject<dyn NSObjectProtocol>> {
    let again = listener.clone();
    let block = RcBlock::new(move |_: NonNull<NSNotification>| {
        again.install();
        if catch_up {
            again.catch_up();
        }
    });
    unsafe {
        NSNotificationCenter::defaultCenter().addObserverForName_object_queue_usingBlock(
            Some(name),
            None,
            None,
            &block,
        )
    }
}

pub fn listen(on: impl Fn(Event) + 'static) -> Option<Listening> {
    let mtm = MainThreadMarker::new()?;
    let listener = Listener::new(mtm, Box::new(on));
    let observers = vec![
        on_launching(
            unsafe { NSApplicationWillFinishLaunchingNotification },
            &listener,
            false,
        ),
        on_launching(
            unsafe { NSApplicationDidFinishLaunchingNotification },
            &listener,
            true,
        ),
    ];
    listener.install();
    Some(Listening {
        _listener: listener,
        _observers: observers,
    })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{Event, GET_URL, read};
    use objc2_core_services::{
        kAEOpenApplication, kAEOpenDocuments, kAEReopenApplication, kCoreEventClass,
        keyAELaunchedAsLogInItem, keyAEPropData, keyDirectObject,
    };
    use objc2_foundation::{NSAppleEventDescriptor, NSString, NSURL};

    #[test]
    fn documents_arrive_as_paths_whether_listed_or_alone() {
        let sent = event(kCoreEventClass, kAEOpenDocuments);
        let one = NSAppleEventDescriptor::descriptorWithFileURL(&NSURL::fileURLWithPath(
            &NSString::from_str("/tmp/one.html"),
        ));
        let two = NSAppleEventDescriptor::descriptorWithFileURL(&NSURL::fileURLWithPath(
            &NSString::from_str("/tmp/two.svg"),
        ));
        let list = NSAppleEventDescriptor::listDescriptor();
        list.insertDescriptor_atIndex(&one, 1);
        list.insertDescriptor_atIndex(&two, 2);
        sent.setParamDescriptor_forKeyword(&list, keyDirectObject);
        assert_eq!(
            read(&sent),
            vec![
                Event::Document(PathBuf::from("/tmp/one.html")),
                Event::Document(PathBuf::from("/tmp/two.svg")),
            ]
        );

        let alone = event(kCoreEventClass, kAEOpenDocuments);
        alone.setParamDescriptor_forKeyword(&one, keyDirectObject);
        assert_eq!(
            read(&alone),
            vec![Event::Document(PathBuf::from("/tmp/one.html"))]
        );

        let as_text = event(kCoreEventClass, kAEOpenDocuments);
        as_text.setParamDescriptor_forKeyword(
            &NSAppleEventDescriptor::descriptorWithString(&NSString::from_str("not a file")),
            keyDirectObject,
        );
        assert!(
            read(&as_text).is_empty(),
            "text that is no file is not a document"
        );
    }

    fn event(class: u32, id: u32) -> objc2::rc::Retained<NSAppleEventDescriptor> {
        NSAppleEventDescriptor::appleEventWithEventClass_eventID_targetDescriptor_returnID_transactionID(
            class, id, None, -1, 0,
        )
    }

    #[test]
    fn a_link_arrives_as_the_text_it_carries() {
        let sent = event(GET_URL, GET_URL);
        let url = NSAppleEventDescriptor::descriptorWithString(&NSString::from_str(
            "https://example.com/a?b=c",
        ));
        sent.setParamDescriptor_forKeyword(&url, keyDirectObject);
        assert_eq!(
            read(&sent),
            vec![Event::Link("https://example.com/a?b=c".to_owned())]
        );
    }

    #[test]
    fn a_link_that_carries_nothing_is_not_an_event() {
        let sent = event(GET_URL, GET_URL);
        assert!(read(&sent).is_empty());
        let empty = NSAppleEventDescriptor::descriptorWithString(&NSString::from_str(""));
        sent.setParamDescriptor_forKeyword(&empty, keyDirectObject);
        assert!(read(&sent).is_empty());
    }

    #[test]
    fn a_launch_says_whether_the_session_started_it() {
        let plain = event(kCoreEventClass, kAEOpenApplication);
        assert_eq!(
            read(&plain),
            vec![Event::Launched {
                as_login_item: false
            }]
        );

        let flag = NSAppleEventDescriptor::descriptorWithEnumCode(keyAELaunchedAsLogInItem);
        plain.setParamDescriptor_forKeyword(&flag, keyAEPropData);
        assert_eq!(
            read(&plain),
            vec![Event::Launched {
                as_login_item: true
            }]
        );
    }

    #[test]
    fn a_reopen_is_reported_as_one() {
        assert_eq!(
            read(&event(kCoreEventClass, kAEReopenApplication)),
            vec![Event::Reopened]
        );
    }

    #[test]
    fn anything_else_is_left_alone() {
        assert!(read(&event(kCoreEventClass, 0x7175_6974)).is_empty());
    }
}
