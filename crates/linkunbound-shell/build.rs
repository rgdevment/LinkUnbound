fn main() {
    slint_build::compile("ui/shell.slint").expect("the interface must compile");

    #[cfg(windows)]
    {
        println!("cargo:rerun-if-changed=linkunbound-shell.manifest");
        embed_manifest::embed_manifest_file("linkunbound-shell.manifest")
            .expect("the manifest must be embedded");
    }
}
