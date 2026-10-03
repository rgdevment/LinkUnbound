#[cfg(not(windows))]
use std::io::Write;
use std::io::{BufRead, BufReader, Read};

#[cfg(not(windows))]
use interprocess::local_socket::Stream;
use interprocess::local_socket::traits::Listener;
#[cfg(not(windows))]
use interprocess::local_socket::traits::Stream as StreamTrait;
use interprocess::local_socket::{ListenerOptions, Name};

/// Pipe names are machine-wide: a user name collides across domain and local accounts and
/// across two sign-ins of one account, the SID and the session do not.
#[cfg(windows)]
fn address() -> String {
    pipe_name(
        linkunbound_win::current_user_sid().as_deref(),
        linkunbound_win::current_session(),
    )
}

#[cfg(windows)]
fn pipe_name(sid: Option<&str>, session: Option<u32>) -> String {
    let user = sid.map_or_else(
        || std::env::var("USERNAME").unwrap_or_else(|_| "default".to_owned()),
        str::to_owned,
    );
    match session {
        Some(session) => format!("linkunbound-{user}-{session}.sock"),
        None => format!("linkunbound-{user}.sock"),
    }
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
/// up durability across a purge rather than privacy. Without it the fallback
/// would be the shared `/tmp`, where another user could have bound the socket
/// first and be handed every link: a directory of our own, made `0700`, is what
/// keeps the fallback private, and one that cannot be made so is not used.
#[cfg(not(windows))]
fn private_fallback() -> Option<std::path::PathBuf> {
    use std::os::unix::fs::PermissionsExt;
    let dir = std::env::temp_dir().join("linkunbound-shell");
    if std::fs::symlink_metadata(&dir).is_ok_and(|m| m.file_type().is_symlink()) {
        return None;
    }
    std::fs::create_dir_all(&dir).ok()?;
    // Only the owner may change the mode, so this succeeding is what says
    // the directory is ours and not one somebody else left at this name.
    std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700)).ok()?;
    let made = std::fs::metadata(&dir).ok()?;
    (made.is_dir() && made.permissions().mode() & 0o777 == 0o700).then(|| dir.join("shell.sock"))
}

#[cfg(not(windows))]
fn address() -> String {
    let kept = linkunbound_core::data_dir().join("shell.sock");
    let chosen = if fits(&kept) {
        kept
    } else {
        private_fallback().unwrap_or(kept)
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

/// Bounded by the clock rather than by a number of turns: on a loaded machine
/// two hundred turns stretched past the five seconds after which a lock reads
/// as abandoned, and the second copy took the room from underneath the first.
#[cfg(not(windows))]
const WAITED_FOR_THE_ROOM: std::time::Duration = std::time::Duration::from_secs(1);

#[cfg(not(windows))]
fn hold(socket: &str) -> Option<Holding> {
    let lock = std::path::Path::new(socket).with_extension("lock");
    let deadline = std::time::Instant::now() + WAITED_FOR_THE_ROOM;
    loop {
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
        if std::time::Instant::now() >= deadline {
            return None;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

#[cfg(not(windows))]
fn owner_only(mode: u32) -> std::fs::Permissions {
    use std::os::unix::fs::PermissionsExt;
    std::fs::Permissions::from_mode(mode)
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

/// A directory another account owns lets that account swap the socket and take every link.
/// Ownership is what decides: a link to a folder of this user's elsewhere is fine, and a
/// file system without Unix modes cannot be closed further, so the chmod is only a best effort.
#[cfg(not(windows))]
fn closed_to_others(dir: &std::path::Path) -> bool {
    use std::os::unix::fs::MetadataExt;
    let _ = std::fs::create_dir_all(dir);
    static PROBES: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    let nth = PROBES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let probe = dir.join(format!(".owner-{}-{nth}", std::process::id()));
    let Ok(made) = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&probe)
        .and_then(|f| f.metadata())
    else {
        return false;
    };
    let _ = std::fs::remove_file(&probe);
    let ours = std::fs::metadata(dir).is_ok_and(|m| m.is_dir() && m.uid() == made.uid());
    if ours {
        let _ = std::fs::set_permissions(dir, owner_only(0o700));
    }
    ours
}

#[cfg(not(windows))]
fn make_room(socket: &str) -> Option<Holding> {
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

/// Opened for identification only: a process that took the pipe's name before the resident did
/// would otherwise be handed this one's token to act as. The crate opens with the default, which
/// allows impersonation, so the Windows client is written by hand.
#[cfg(windows)]
fn hand_over_at(socket: &str, url: &str) -> bool {
    linkunbound_win::write_line_to_pipe(&format!(r"\\.\pipe\{socket}"), &url.replace('\n', ""))
}

#[cfg(not(windows))]
fn hand_over_at(socket: &str, url: &str) -> bool {
    let Some(name) = named(socket) else {
        return false;
    };
    let Ok(mut stream) = Stream::connect(name) else {
        return false;
    };
    writeln!(stream, "{}", url.replace('\n', "")).is_ok()
}

/// This user and the system, nobody else: the pipe's default descriptor lets every account on
/// the machine read it. Labelled low so a sandboxed browser can still hand a link across. Owned
/// by the user even when elevated, whose default owner is Administrators: the client checks it.
#[cfg(windows)]
fn guarded<'a>(options: ListenerOptions<'a>, sid: Option<String>) -> Option<ListenerOptions<'a>> {
    use interprocess::os::windows::local_socket::ListenerOptionsExt;
    use interprocess::os::windows::security_descriptor::SecurityDescriptor;
    let sid = sid?;
    let sddl = format!("O:{sid}D:(A;;GA;;;SY)(A;;GA;;;{sid})S:(ML;;NW;;;LW)");
    let wide = widestring::U16CString::from_str(&sddl).ok()?;
    let descriptor = SecurityDescriptor::deserialize(&wide).ok()?;
    Some(options.security_descriptor(descriptor))
}

#[cfg(windows)]
fn only_this_user(options: ListenerOptions<'_>) -> Option<ListenerOptions<'_>> {
    guarded(options, linkunbound_win::current_user_sid())
}

#[cfg(not(windows))]
fn only_this_user(options: ListenerOptions<'_>) -> Option<ListenerOptions<'_>> {
    Some(options)
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

/// A listener failing for good would otherwise spin a core for the lifetime of
/// a process meant to sit in memory for days.
fn gave_up(refused: u32) -> bool {
    refused > 100
}

/// A resident nobody can reach is worse than none: it keeps the tray icon and answers nothing,
/// while the next click, finding the name free, starts a second one beside it. This one leaves,
/// and the next click starts the one that works.
fn leave_deaf() {
    std::process::exit(0);
}

fn claim_at(
    socket: &str,
    arrived: impl Fn(String) + Send + Sync + 'static,
) -> Option<std::thread::JoinHandle<()>> {
    let arrived = std::sync::Arc::new(arrived);
    #[cfg(not(windows))]
    let _room = {
        let parent = std::path::Path::new(socket).parent()?;
        if !closed_to_others(parent) {
            return None;
        }
        make_room(socket)
    };
    let name = named(socket)?;
    let listener = only_this_user(ListenerOptions::new().name(name))?
        .create_sync()
        .ok()?;
    // The socket file takes its mode from the umask; a permissive one lets other accounts write.
    #[cfg(not(windows))]
    let _ = std::fs::set_permissions(socket, owner_only(0o600));

    Some(std::thread::spawn(move || {
        let mut refused = 0u32;
        loop {
            let Ok(stream) = listener.accept() else {
                refused += 1;
                if gave_up(refused) {
                    leave_deaf();
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

    /// A directory of the tests' own: claiming closes the socket's parent to other accounts.
    #[cfg(not(windows))]
    fn scratch(name: &str) -> String {
        let dir = std::env::temp_dir().join(format!("linkunbound-test-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        dir.join(format!("{name}.sock"))
            .to_string_lossy()
            .into_owned()
    }

    #[test]
    #[cfg(windows)]
    fn the_pipe_is_named_after_the_account_and_the_session_not_the_user_name() {
        assert_eq!(
            super::pipe_name(Some("S-1-5-21-1-2-3-1001"), Some(2)),
            "linkunbound-S-1-5-21-1-2-3-1001-2.sock"
        );
        assert_ne!(
            super::pipe_name(Some("S-1-5-21-1-2-3-1001"), Some(1)),
            super::pipe_name(Some("S-1-5-21-1-2-3-1001"), Some(2)),
            "one account signed in twice keeps two residents apart"
        );
        assert!(super::address().starts_with("linkunbound-S-1-"));
    }

    /// Another account can take the name before the resident does. Its pipe answers, but the
    /// link must not be written to it: a pipe this user does not own reads as nobody there.
    #[test]
    #[cfg(windows)]
    fn a_pipe_this_user_does_not_own_is_not_handed_the_link() {
        use interprocess::local_socket::{GenericNamespaced, ListenerOptions, ToNsName};
        use interprocess::os::windows::local_socket::ListenerOptionsExt;
        use interprocess::os::windows::security_descriptor::SecurityDescriptor;

        let name = scratch("squatted");
        let wide = widestring::U16CString::from_str("O:BAD:(A;;GA;;;WD)").expect("sddl");
        let descriptor = SecurityDescriptor::deserialize(&wide).expect("descriptor");
        let Ok(_squatter) = ListenerOptions::new()
            .name(
                name.as_str()
                    .to_ns_name::<GenericNamespaced>()
                    .expect("a name"),
            )
            .security_descriptor(descriptor)
            .create_sync()
        else {
            // Administrators can only own what an elevated token creates.
            assert!(
                std::env::var_os("GITHUB_ACTIONS").is_none(),
                "CI runs elevated, so the squatter must be created there"
            );
            eprintln!("skipped: not elevated, no pipe owned by another account");
            return;
        };
        assert!(!hand_over_at(&name, "https://example.test/"));
    }

    #[test]
    #[cfg(windows)]
    fn a_channel_that_cannot_be_guarded_is_not_opened() {
        use interprocess::local_socket::{GenericNamespaced, ListenerOptions, ToNsName};

        let name = scratch("guard");
        let named = name.to_ns_name::<GenericNamespaced>().expect("a name");
        assert!(
            super::guarded(ListenerOptions::new().name(named.clone()), None).is_none(),
            "no account to name means no channel"
        );
        assert!(
            super::guarded(
                ListenerOptions::new().name(named.clone()),
                Some("not-a-sid".to_owned())
            )
            .is_none(),
            "a descriptor the system refuses means no channel"
        );
        assert!(
            super::guarded(
                ListenerOptions::new().name(named),
                linkunbound_win::current_user_sid()
            )
            .is_some(),
            "this account's own sid builds one"
        );
    }

    #[test]
    fn a_link_with_a_pipe_in_it_arrives_whole() {
        let socket = scratch("pipe");
        let (tx, rx) = channel();
        let _server = claim_at(&socket, move |url| {
            let _ = tx.send(url);
        })
        .expect("the socket can be claimed");
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
        let _server = claim_at(&socket, |_| {}).expect("the socket can be claimed");
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
        let _server = claim_at(&socket, move |url| {
            let _ = tx.send(url);
        })
        .expect("the socket can be claimed");

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
        let _server = claim_at(&socket, move |url| {
            let _ = tx.send(url);
        })
        .expect("the socket can be claimed");

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

        let got = rx
            .recv_timeout(Duration::from_secs(5))
            .expect("the capped line still arrives");
        assert!(
            got.len() as u64 <= super::LONGEST_LINK,
            "read {} bytes, past the cap",
            got.len()
        );
    }

    /// Chrome follows links of about 32 KB, and the 1.x pipe broke at 4 KB with a real Teams
    /// link: the cap has to sit far above both, and one of that size has to arrive whole.
    #[test]
    fn a_link_as_long_as_a_browser_follows_arrives_whole() {
        let socket = scratch("long");
        let (tx, rx) = channel();
        let _server = claim_at(&socket, move |url| {
            let _ = tx.send(url);
        })
        .expect("the socket can be claimed");
        let sent = format!("https://teams.test/l/?p={}", "x".repeat(32 * 1024));
        assert!(hand_over_at(&socket, &sent));
        assert_eq!(
            rx.recv_timeout(Duration::from_secs(5)).as_deref(),
            Ok(sent.as_str())
        );
        assert!(super::LONGEST_LINK >= 4 * (sent.len() as u64));
    }

    #[test]
    fn a_listener_that_keeps_failing_is_given_up_on_only_after_a_while() {
        assert!(!super::gave_up(0));
        assert!(!super::gave_up(100));
        assert!(super::gave_up(101));
    }

    #[cfg(not(windows))]
    #[test]
    fn the_fallback_socket_lives_in_a_directory_nobody_else_can_open() {
        use std::os::unix::fs::PermissionsExt;
        let fallback = super::private_fallback().expect("a private place");
        let dir = fallback.parent().expect("a directory");
        let mode = std::fs::metadata(dir).expect("made").permissions().mode();
        assert_eq!(mode & 0o777, 0o700, "{mode:o}");
        assert!(fallback.ends_with("shell.sock"));
        assert!(super::fits(&fallback));
    }

    #[cfg(not(windows))]
    #[test]
    fn the_socket_and_its_directory_are_closed_to_other_accounts() {
        use std::os::unix::fs::PermissionsExt;
        let dir =
            std::env::temp_dir().join(format!("linkunbound-test-mode-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o755)).expect("open dir");
        let socket = dir.join("shell.sock").to_string_lossy().into_owned();

        let _server = claim_at(&socket, |_| {}).expect("the socket can be claimed");

        let mode = |p: &str| std::fs::metadata(p).expect("there").permissions().mode() & 0o777;
        assert_eq!(mode(&socket), 0o600, "socket {:o}", mode(&socket));
        assert_eq!(mode(&dir.to_string_lossy()), 0o700);
        let _ = std::fs::remove_file(&socket);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A data folder moved elsewhere and linked back is still this user's, and must keep working.
    #[cfg(not(windows))]
    #[test]
    fn a_socket_directory_linked_to_a_folder_of_this_user_is_used() {
        let base = std::path::PathBuf::from(scratch("elsewhere")).with_extension("");
        let real = base.with_extension("real");
        let link = base.with_extension("link");
        let _ = std::fs::create_dir_all(&real);
        let _ = std::fs::remove_file(&link);
        std::os::unix::fs::symlink(&real, &link).expect("a link");
        let socket = link.join("shell.sock").to_string_lossy().into_owned();

        let server = claim_at(&socket, |_| {});
        assert!(server.is_some());
        drop(server);
        let _ = std::fs::remove_file(&socket);
        let _ = std::fs::remove_file(&link);
        let _ = std::fs::remove_dir_all(&real);
    }

    /// A directory owned by another account, here root's `/`, is refused rather than used.
    #[cfg(not(windows))]
    #[test]
    fn a_socket_directory_this_user_cannot_own_is_not_used() {
        assert!(!super::closed_to_others(std::path::Path::new("/")));
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
        assert!(
            super::WAITED_FOR_THE_ROOM * 4 < super::HELD_FOR_LONG_ENOUGH,
            "a lock held by a copy still waiting for the room must never read as abandoned"
        );
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

    /// A lock left by a copy that died mid-claim would otherwise keep every resident out for
    /// good; one old enough to be nobody's is taken over, one fresh enough is not.
    #[cfg(not(windows))]
    #[test]
    fn a_lock_nobody_has_held_for_a_while_is_taken_over() {
        let socket = scratch("stale");
        let lock = std::path::Path::new(&socket).with_extension("lock");
        std::fs::write(&lock, b"").expect("a lock left behind");
        let long_ago = std::time::SystemTime::now() - Duration::from_secs(60);
        std::fs::File::options()
            .write(true)
            .open(&lock)
            .and_then(|f| f.set_modified(long_ago))
            .expect("an old lock");
        let taken = super::hold(&socket).expect("an abandoned lock is nobody's");
        drop(taken);
        assert!(!lock.exists(), "let go, the lock is gone");
    }

    /// The other half of the same decision: one that does answer belongs to a resident that is
    /// still running, and taking it away would strand every link it was about to be handed.
    #[cfg(not(windows))]
    #[test]
    fn a_socket_a_living_resident_still_answers_on_is_left_alone() {
        let socket = scratch("held");
        let (tx, rx) = channel();
        let _server = claim_at(&socket, move |url| {
            let _ = tx.send(url);
        })
        .expect("the socket can be claimed");

        assert!(
            claim_at(&socket, |_| {}).is_none(),
            "the second copy must not take a socket the first is answering on"
        );

        let sent = "https://example.test/still-listening";
        assert!(hand_over_at(&socket, sent));
        assert_eq!(rx.recv_timeout(Duration::from_secs(3)).as_deref(), Ok(sent));
    }
}
