use std::io::{BufRead, BufReader, Write};
use std::sync::mpsc::Sender;

use interprocess::local_socket::traits::{Listener, Stream as StreamTrait};
use interprocess::local_socket::{GenericNamespaced, ListenerOptions, Stream, ToNsName};

/// One socket per user session, so two people on one machine never cross links.
fn address() -> String {
    let session = std::env::var("USERNAME").unwrap_or_else(|_| "default".to_owned());
    format!("linkunbound-{session}.sock")
}

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

#[must_use]
pub fn claim(links: Sender<String>) -> Option<std::thread::JoinHandle<()>> {
    claim_at(&address(), links)
}

fn claim_at(socket: &str, links: Sender<String>) -> Option<std::thread::JoinHandle<()>> {
    let name = socket.to_ns_name::<GenericNamespaced>().ok()?;
    let listener = ListenerOptions::new().name(name).create_sync().ok()?;

    Some(std::thread::spawn(move || {
        loop {
            let Ok(stream) = listener.accept() else {
                continue;
            };
            let mut line = String::new();
            if BufReader::new(stream).read_line(&mut line).is_ok() {
                let url = line.trim_end().to_owned();
                if !url.is_empty() && links.send(url).is_err() {
                    return;
                }
            }
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
        let Some(_server) = claim_at(&socket, tx) else {
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
        let (tx, _rx) = channel();
        let Some(_server) = claim_at(&socket, tx) else {
            return;
        };
        let (other, _) = channel();
        assert!(claim_at(&socket, other).is_none());
    }

    #[test]
    fn handing_over_with_nobody_listening_says_so_instead_of_hanging() {
        assert!(!hand_over_at(&scratch("nobody"), "https://example.test/"));
    }
}
