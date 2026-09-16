// Never a console program, in debug either: the tray, the Start menu and Explorer all opened one
// beside the window. What a debug build wants from a console — its prints and its panics — it gets
// by borrowing the terminal that started it, when one did.
#![windows_subsystem = "windows"]

fn main() {
    #[cfg(all(windows, debug_assertions))]
    linkunbound_win::attach_parent_console();
    linkunbound_lib::run();
}
