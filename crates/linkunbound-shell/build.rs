fn main() {
    slint_build::compile("ui/shell.slint").expect("the interface must compile");

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
            .compile()
            .expect("the icon must be embedded");
    }
}
