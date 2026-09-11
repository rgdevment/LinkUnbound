use std::io::{BufRead, BufReader, Read, Write};

use interprocess::local_socket::traits::{Listener, Stream as StreamTrait};
use interprocess::local_socket::{GenericNamespaced, ListenerOptions, Stream, ToNsName};

/// One socket per user session, so two people on one machine never cross links.
fn address() -> String {
    let session = std::env::var("USERNAME").unwrap_or_else(|_| "default".to_owned());
    format!("linkunbound-{session}.sock")
}

/// Chrome stops at about 32 KB and the 1.x pipe broke at 4 KB with a real
/// Teams link, so this sits far above anything a browser would follow.
const LONGEST_LINK: u64 = 256 * 1024;

/// Read to the end of the line, never split: `|` is legal inside a URL.
pub fn hand_over(url: &str) -> bool {
    hand_over_at(&address(), url)
}

fn hand_over_at(socket: &str, url: &str) -> bool {
    let Ok(name) = socket.to_ns_name::<GenericNamespaced>() else {
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
    let name = socket.to_ns_name::<GenericNamespaced>().ok()?;
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
    fn scratch(name: &str) -> String {
        format!("linkunbound-test-{name}-{}.sock", std::process::id())
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
        use interprocess::local_socket::traits::Stream as _;
        use interprocess::local_socket::{GenericNamespaced, Stream, ToNsName};

        let socket = scratch("mute");
        let (tx, rx) = channel();
        let Some(_server) = claim_at(&socket, move |url| {
            let _ = tx.send(url);
        }) else {
            return;
        };

        let name = socket
            .clone()
            .to_ns_name::<GenericNamespaced>()
            .expect("should name");
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
        use interprocess::local_socket::traits::Stream as _;
        use interprocess::local_socket::{GenericNamespaced, Stream, ToNsName};
        use std::io::Write as _;

        let socket = scratch("flood");
        let (tx, rx) = channel();
        let Some(_server) = claim_at(&socket, move |url| {
            let _ = tx.send(url);
        }) else {
            return;
        };

        let name = socket
            .clone()
            .to_ns_name::<GenericNamespaced>()
            .expect("should name");
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
}
