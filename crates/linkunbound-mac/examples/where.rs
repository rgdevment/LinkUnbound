//! `cargo run -p linkunbound-mac --example where`

#[cfg(target_os = "macos")]
fn main() {
    println!("cursor:    {:?}", linkunbound_mac::cursor());
    if let Some((x, y)) = linkunbound_mac::cursor() {
        println!("work area: {:?}", linkunbound_mac::work_area_at(x, y));
    }
    println!("shift:     {}", linkunbound_mac::shift_is_down());
    println!("source:    {:?}", linkunbound_mac::source_app());
    println!("light:     {}", linkunbound_mac::windows_are_light());
}

#[cfg(not(target_os = "macos"))]
fn main() {}
