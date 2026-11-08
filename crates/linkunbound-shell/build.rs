fn main() {
    // Tests find the picker's buttons by id; a release build carries no ids it would never use.
    let debug = std::env::var("PROFILE").as_deref() == Ok("debug");
    slint_build::compile_with_config(
        "ui/shell.slint",
        slint_build::CompilerConfiguration::new().with_debug_info(debug),
    )
    .expect("the interface must compile");

    #[cfg(windows)]
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        println!("cargo:rerun-if-changed=linkunbound-shell.manifest");
        embed_manifest::embed_manifest_file("linkunbound-shell.manifest")
            .expect("the manifest must be embedded");

        // The executable's own icon: without it Alt+Tab, the task switcher and Explorer show the
        // blank one for the very window the person sees most.
        println!("cargo:rerun-if-changed=../../app/src-tauri/icons/icon.ico");
        tauri_winres::WindowsResource::new()
            .set_icon("../../app/src-tauri/icons/icon.ico")
            // Browsers name the app in their "Open …?" prompt from these, not from the registry.
            .set("ProductName", "LinkUnbound")
            .set("FileDescription", "LinkUnbound")
            .compile()
            .expect("the icon must be embedded");
    }
}
