# Vendored crates

Third-party crates kept here with a change LinkUnbound needs and upstream does not have yet.
They are not workspace members (`exclude = ["vendor"]`), so fmt, clippy, tests, the line
ceiling and SonarCloud leave them alone. They reach the build only through `[patch.crates-io]`
in the root `Cargo.toml`.

## softbuffer 0.4.8

**Why:** on macOS, softbuffer hands every frame to Core Graphics as
`CGImageAlphaInfo::NoneSkipFirst`, which reads the alpha byte as padding: every pixel lands
opaque. Slint's software renderer does write premultiplied alpha into that byte, so with it
dropped nothing behind the picker's view can show through, and the Liquid Glass or vibrancy
view under it stays hidden.

**What changed:** one line in `src/backends/cg.rs`, marked `LinkUnbound:`, from
`NoneSkipFirst` to `PremultipliedFirst`. Everything else is the crate as published on
crates.io (`.cargo_vcs_info.json` names the upstream commit), minus the files only its own
CI used. Windows and the other backends are untouched.

**Licence:** MIT OR Apache-2.0, both texts beside it. It was already listed in the
third-party notices as a dependency of Slint.

**When it goes:** once a released softbuffer keeps the alpha on macOS and Slint depends on
it. Then delete `vendor/softbuffer` and the `[patch.crates-io]` entry, and check that
`cargo run -p linkunbound-shell --example preview -- --glass` on a Mac still shows the
desktop blurred behind the picker.

**Updating Slint meanwhile:** if Slint moves to a softbuffer other than 0.4.8, the patch
stops applying and Cargo warns that it is unused. Copy the new version here, carry the
same line over, and update this note.
