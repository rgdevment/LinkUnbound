use std::io::{BufRead, BufReader, Read, Write};

use interprocess::local_socket::traits::{Listener, Stream as StreamTrait};
use interprocess::local_socket::{ListenerOptions, Name, Stream};

/// One socket per user session, so two people on one machine never cross links.
#[cfg(windows)]
fn address() -> String {
    let session = std::env::var("USERNAME").unwrap_or_else(|_| "default".to_owned());
    format!("linkunbound-{session}.sock")
}

/// `sun_path` holds this many bytes and not one more. A long enough user name
/// pushes the durable path past it, and the resident then fails to bind with
/// nothing on screen to say why.
#[cfg(not(windows))]
const LONGEST_SOCKET_PATH: usize = 103;

#[cfg(not(windows))]
fn fits(path: &std::path::Path) -> bool {
    path.as_os_str().as_encoded_bytes().len() <= LONGEST_SOCKET_PATH
}

/// `$TMPDIR` is this user's own and kept at `drwx------`, so the fallback gives
/// up durability across a purge rather than privacy.
#[cfg(not(windows))]
fn address() -> String {
    let kept = linkunbound_core::data_dir().join("shell.sock");
    let chosen = if fits(&kept) {
        kept
    } else {
        std::env::temp_dir().join("linkunbound-shell.sock")
    };
    chosen.to_string_lossy().into_owned()
}

#[cfg(windows)]
fn named(socket: &str) -> Option<Name<'_>> {
    use interprocess::local_socket::{GenericNamespaced, ToNsName};
    socket.to_ns_name::<GenericNamespaced>().ok()
}

#[cfg(not(windows))]
fn named(socket: &str) -> Option<Name<'_>> {
    use interprocess::local_socket::{GenericFilePath, ToFsName};
    socket.to_fs_name::<GenericFilePath>().ok()
}

/// Between reading that nobody answers and taking the file away, another copy
/// can have bound a new socket at that path: both would then read as the
/// resident, and the machine would carry two of them with two tray icons.
#[cfg(not(windows))]
struct Holding(std::path::PathBuf);

#[cfg(not(windows))]
impl Drop for Holding {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

#[cfg(not(windows))]
const HELD_FOR_LONG_ENOUGH: std::time::Duration = std::time::Duration::from_secs(5);

#[cfg(not(windows))]
fn abandoned(held: std::time::Duration) -> bool {
    held > HELD_FOR_LONG_ENOUGH
}

#[cfg(not(windows))]
fn hold(socket: &str) -> Option<Holding> {
    let lock = std::path::Path::new(socket).with_extension("lock");
    for _ in 0..200 {
        if std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&lock)
            .is_ok()
        {
            return Some(Holding(lock));
        }
        if std::fs::metadata(&lock)
            .and_then(|m| m.modified())
            .is_ok_and(|held| abandoned(held.elapsed().unwrap_or_default()))
        {
            let _ = std::fs::remove_file(&lock);
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    None
}

#[cfg(not(windows))]
fn clear(socket: &str) {
    let path = std::path::Path::new(socket);
    if !path.exists() {
        return;
    }
    if named(socket).is_some_and(|name| Stream::connect(name).is_ok()) {
        return;
    }
    let _ = std::fs::remove_file(path);
}

#[cfg(not(windows))]
fn make_room(socket: &str) -> Option<Holding> {
    if let Some(parent) = std::path::Path::new(socket).parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    // Without the lock nothing may be taken away: binding over whatever is
    // there is safe, clearing somebody else's socket is not.
    let held = hold(socket)?;
    clear(socket);
    Some(held)
}

/// Chrome stops at about 32 KB and the 1.x pipe broke at 4 KB with a real
/// Teams link, so this sits far above anything a browser would follow.
const LONGEST_LINK: u64 = 256 * 1024;

/// Read to the end of the line, never split: `|` is legal inside a URL.
pub fn hand_over(url: &str) -> bool {
    hand_over_at(&address(), url)
}

fn hand_over_at(socket: &str, url: &str) -> bool {
    let Some(name) = named(socket) else {
        return false;
    };
    let Ok(mut stream) = Stream::connect(name) else {
        return false;
    };
    writeln!(stream, "{}", url.replace('\n', "")).is_ok()
}

/// Hands each link to `arrived` on the listener thread. A callback rather than a
/// channel so the caller can wake its event loop the moment one lands, instead
/// of finding it on the next poll.
#[must_use]
pub fn claim(
    arrived: impl Fn(String) + Send + Sync + 'static,
) -> Option<std::thread::JoinHandle<()>> {
    claim_at(&address(), arrived)
}

fn claim_at(
    socket: &str,
    arrived: impl Fn(String) + Send + Sync + 'static,
) -> Option<std::thread::JoinHandle<()>> {
    let arrived = std::sync::Arc::new(arrived);
    #[cfg(not(windows))]
    let _room = make_room(socket);
    let name = named(socket)?;
    let listener = ListenerOptions::new().name(name).create_sync().ok()?;

    Some(std::thread::spawn(move || {
        let mut refused = 0u32;
        loop {
            let Ok(stream) = listener.accept() else {
                // A listener failing for good would otherwise spin a core for
                // the lifetime of a process meant to sit in memory for days.
                refused += 1;
                if refused > 100 {
                    return;
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
                continue;
            };
            refused = 0;
            // Read on a thread of its own: any process of this user can connect
            // and never send a line, and reading inline wedged the accept loop
            // for good — one silent peer cost every link that followed.
            let hand = std::sync::Arc::clone(&arrived);
            let _ = std::thread::Builder::new().spawn(move || {
                let mut line = String::new();
                // Capped: a caller sending a gigabyte with no newline would
                // otherwise be read whole into a process that lives all day.
                let mut capped = BufReader::new(stream).take(LONGEST_LINK);
                if capped.read_line(&mut line).is_ok() {
                    let url = line.trim_end().to_owned();
                    if !url.is_empty() {
                        hand(url);
                    }
                }
            });
        }
    }))
}

#[cfg(test)]
mod tests {
    use super::{claim_at, hand_over_at};
    use std::sync::mpsc::channel;
    use std::time::Duration;

    /// Tests run at once, so a shared name would let one decide for the rest.
    #[cfg(windows)]
    fn scratch(name: &str) -> String {
        format!("linkunbound-test-{name}-{}.sock", std::process::id())
    }

    #[cfg(not(windows))]
    fn scratch(name: &str) -> String {
        std::env::temp_dir()
            .join(format!(
                "linkunbound-test-{name}-{}.sock",
                std::process::id()
            ))
            .to_string_lossy()
            .into_owned()
    }

    #[test]
    fn a_link_with_a_pipe_in_it_arrives_whole() {
        let socket = scratch("pipe");
        let (tx, rx) = channel();
        let Some(_server) = claim_at(&socket, move |url| {
            let _ = tx.send(url);
        }) else {
            return;
        };
        let sent = "https://intranet.corp/informes?filtro=activo|urgente&b=1|2";
        assert!(hand_over_at(&socket, sent));
        let got = rx
            .recv_timeout(Duration::from_secs(3))
            .expect("nothing arrived");
        assert_eq!(got, sent);
    }

    #[test]
    fn a_socket_of_this_user_can_be_named() {
        assert!(super::named(&scratch("named")).is_some());
    }

    #[test]
    fn the_second_process_cannot_claim_what_the_first_holds() {
        let socket = scratch("twice");
        let Some(_server) = claim_at(&socket, |_| {}) else {
            return;
        };
        assert!(claim_at(&socket, |_| {}).is_none());
    }

    /// Any process of this user can open the socket. One that connects and
    /// never writes used to wedge the accept loop, and from that moment no link
    /// reached the picker at all.
    #[test]
    fn a_caller_that_connects_and_says_nothing_does_not_block_the_next_link() {
        use interprocess::local_socket::Stream;
        use interprocess::local_socket::traits::Stream as _;

        let socket = scratch("mute");
        let (tx, rx) = channel();
        let Some(_server) = claim_at(&socket, move |url| {
            let _ = tx.send(url);
        }) else {
            return;
        };

        let name = super::named(&socket).expect("should name");
        let _mute = Stream::connect(name).expect("should connect");

        let sent = "https://example.test/after-the-mute-one";
        assert!(hand_over_at(&socket, sent));
        assert_eq!(
            rx.recv_timeout(Duration::from_secs(3)).as_deref(),
            Ok(sent),
            "a silent caller must not cost every link that follows"
        );
    }

    /// The socket takes a line from any process of this user. Without a cap,
    /// one of them could hand a gigabyte to a process that stays in memory all
    /// day, and it would be read whole.
    #[test]
    fn a_caller_sending_far_too_much_does_not_grow_the_resident_without_bound() {
        use interprocess::local_socket::Stream;
        use interprocess::local_socket::traits::Stream as _;
        use std::io::Write as _;

        let socket = scratch("flood");
        let (tx, rx) = channel();
        let Some(_server) = claim_at(&socket, move |url| {
            let _ = tx.send(url);
        }) else {
            return;
        };

        let name = super::named(&socket).expect("should name");
        let mut flood = Stream::connect(name).expect("should connect");
        let chunk = "x".repeat(64 * 1024);
        for _ in 0..16 {
            if flood.write_all(chunk.as_bytes()).is_err() {
                break;
            }
        }
        let _ = flood.write_all(
            b"
",
        );
        drop(flood);

        if let Ok(got) = rx.recv_timeout(Duration::from_secs(5)) {
            assert!(
                got.len() as u64 <= super::LONGEST_LINK,
                "read {} bytes, past the cap",
                got.len()
            );
        }
    }

    #[test]
    fn handing_over_with_nobody_listening_says_so_instead_of_hanging() {
        assert!(!hand_over_at(&scratch("nobody"), "https://example.test/"));
    }

    /// A pipe goes with the process that held it; a socket file does not. To the launch that
    /// follows a resident somebody killed, what is left is a path that exists and answers
    /// nothing — and it used to refuse the claim, so the picker never came back and every link
    /// after it went nowhere in silence.
    #[cfg(not(windows))]
    #[test]
    fn a_socket_left_behind_by_a_resident_that_died_does_not_lock_the_next_one_out() {
        let socket = scratch("orphan");
        let path = std::path::Path::new(&socket);
        let _ = std::fs::remove_file(path);
        std::fs::write(path, b"").expect("something where the socket was");

        let (tx, rx) = channel();
        let server = claim_at(&socket, move |url| {
            let _ = tx.send(url);
        });
        assert!(
            server.is_some(),
            "what nobody answers on is nobody's to keep"
        );

        let sent = "https://example.test/after-the-orphan";
        assert!(hand_over_at(&socket, sent));
        assert_eq!(rx.recv_timeout(Duration::from_secs(3)).as_deref(), Ok(sent));

        drop(server);
        let _ = std::fs::remove_file(path);
    }

    /// `sun_path` holds 104 bytes. The durable path is 58 characters plus the user's name, so a
    /// long enough one pushed it past the limit and the resident failed to bind with nothing on
    /// screen: no picker, no links, no explanation.
    #[cfg(not(windows))]
    #[test]
    fn a_path_too_long_for_the_system_is_not_the_one_asked_for() {
        use std::path::Path;

        let durable =
            |who: &str| format!("/Users/{who}/Library/Application Support/LinkUnbound/shell.sock");

        assert!(super::fits(Path::new(&durable("ana"))));
        assert!(
            super::fits(Path::new(&durable(&"u".repeat(45)))),
            "forty-five still fits, which is where the room runs out"
        );
        assert!(
            !super::fits(Path::new(&durable(&"u".repeat(46)))),
            "and one more does not"
        );

        let exactly = "x".repeat(super::LONGEST_SOCKET_PATH);
        assert!(
            super::fits(Path::new(&exactly)),
            "on the mark it still fits"
        );
        assert!(!super::fits(Path::new(&format!("{exactly}x"))));
    }

    /// Whatever the name it lands on, it has to be one the system can bind, or the choice
    /// merely moves the failure.
    #[cfg(not(windows))]
    #[test]
    fn whatever_address_is_chosen_is_one_that_fits() {
        let chosen = super::address();
        assert!(super::fits(std::path::Path::new(&chosen)));
        assert!(chosen.ends_with(".sock"), "{chosen}");
        assert!(std::path::Path::new(&chosen).is_absolute(), "{chosen}");
    }

    /// A lock nobody can release is worse than the race it prevents: a copy that died holding
    /// it would lock every resident out for good.
    #[cfg(not(windows))]
    #[test]
    fn a_lock_is_taken_over_only_once_it_is_plainly_abandoned() {
        use std::time::Duration;
        assert!(!super::abandoned(Duration::ZERO));
        assert!(!super::abandoned(super::HELD_FOR_LONG_ENOUGH));
        assert!(super::abandoned(
            super::HELD_FOR_LONG_ENOUGH + Duration::from_millis(1)
        ));
    }

    /// Two copies starting at once over one orphan both read as the resident, and the machine
    /// carried two of them with two tray icons. Only one may hold the room at a time.
    #[cfg(not(windows))]
    #[test]
    fn only_one_copy_at_a_time_may_take_the_socket_away() {
        let socket = scratch("room");
        let path = std::path::Path::new(&socket);
        let _ = std::fs::remove_file(path);
        std::fs::write(path, b"").expect("an orphan to clear");

        let held = super::make_room(&socket).expect("the first one takes the room");
        assert!(
            super::make_room(&socket).is_none(),
            "the second must wait rather than clear underneath the first"
        );
        drop(held);
        assert!(
            super::make_room(&socket).is_some(),
            "and the room is free again once it is let go"
        );
        let _ = std::fs::remove_file(path);
        let _ = std::fs::remove_file(path.with_extension("lock"));
    }

    /// The other half of the same decision: one that does answer belongs to a resident that is
    /// still running, and taking it away would strand every link it was about to be handed.
    #[cfg(not(windows))]
    #[test]
    fn a_socket_a_living_resident_still_answers_on_is_left_alone() {
        let socket = scratch("held");
        let (tx, rx) = channel();
        let Some(_server) = claim_at(&socket, move |url| {
            let _ = tx.send(url);
        }) else {
            return;
        };

        assert!(
            claim_at(&socket, |_| {}).is_none(),
            "the second copy must not take a socket the first is answering on"
        );

        let sent = "https://example.test/still-listening";
        assert!(hand_over_at(&socket, sent));
        assert_eq!(rx.recv_timeout(Duration::from_secs(3)).as_deref(), Ok(sent));
    }
}
