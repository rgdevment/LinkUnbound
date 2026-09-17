# Contributing to LinkUnbound

Thank you for considering contributing to **LinkUnbound**. Whether it's your first open source contribution or you're experienced — everyone is welcome.

---

## Philosophy

**LinkUnbound** is a personal project shared with the community. It was created by a single developer ([@rgdevment](https://github.com/rgdevment)) to solve a daily frustration: choosing which browser opens a link shouldn't require a computer science degree.

There is no premium version and no feature is ever held back: what you install is the whole application. LinkUnbound is free software, built for and shared with the community.

It is also dual licensed. Anyone who wants to redistribute it inside a product of their own needs [separate terms](COMMERCIAL.md) — that is what funds the time spent on it, and it never comes at the expense of the free edition.

We believe in:

- **Simplicity** — Features that matter, no bloat.
- **Privacy first** — Your data stays local, always.
- **Performance** — Lightweight and fast. The picker should feel instant.
- **Collaboration** — We build together.

---

## How Can I Contribute?

### Share Feedback

- **Use the app** — The most valuable feedback comes from real users.
- **Report issues** — If something doesn't work, [let us know](https://github.com/rgdevment/LinkUnbound/issues).
- **Suggest improvements** — Have an idea? [Open an issue](https://github.com/rgdevment/LinkUnbound/issues/new).

### Report Bugs

1. Search [existing issues](https://github.com/rgdevment/LinkUnbound/issues) first.
2. If it's new, open an issue with:
   - Clear description of the problem
   - Steps to reproduce
   - LinkUnbound version and OS version (Windows 10/11 or macOS)
   - Screenshots if applicable

### Contribute Code

1. **Fork** the repository.
2. **Choose the base branch:**
   - `main` — active development. Version 2.0 lands here.
   - `v1-stable` — the frozen 1.4.x line. Security and crash fixes only, no new features.
3. **Create a branch** from it (`git checkout -b feature/my-improvement`).
4. **Make your changes** following the style guide below.
5. **Run checks:**

   ```sh
   cargo fmt --all
   cargo clippy --workspace --all-targets
   cargo test --workspace
   npm run lint --prefix app && npm test --prefix app
   ```

6. **Open a Pull Request** to the branch you started from.

### Translate

LinkUnbound supports English and Spanish. Help bring it to more languages — see the [localization section](#adding-a-translation) below.

### Improve Documentation

Found something confusing? Missing information? PRs welcome.

---

## Project Structure

LinkUnbound is a Cargo workspace. Two binaries ship in one package: the
resident, which owns the tray, the picker and the notice, and the settings
window, which opens on demand.

```text
LinkUnbound/
  crates/
    linkunbound-core/     # Pure Rust: URLs, rules, browsers, preferences, the update feed
    linkunbound-win/      # Windows: registration, detection, icons, start-up, the Win32 calls
    linkunbound-mac/      # macOS: Launch Services, Apple Events, icons, login item (objc2)
    linkunbound-shell/    # The resident: Slint picker and notice, tray, single instance
      ui/shell.slint      # The picker in its two looks, and the notice
      examples/preview.rs # The picker on its own, for design work
  app/
    src/                  # The settings window, in React
    src-tauri/            # Its Tauri side: commands, health, updates, the Store
      capabilities/       # What the front end is allowed to call
      nsis/hooks.nsh      # The Windows installer's hooks (1.x removal, registration)
      msix/               # The Store package manifest
  scripts/                # sidecar.sh builds the resident for the bundler; third-party.mjs the notices
  .github/workflows/      # ci, rules, bundle, release, feed, mutants
```

**Key conventions:**

- Business logic lives in `crates/linkunbound-core`. It depends on neither
  Tauri nor any platform API, and CI enforces that each crate builds alone.
- The core never prints. The picker would inherit the output as garbage.
- `unsafe` is forbidden workspace-wide. The platform crates declare
  `allow(unsafe_code)` in a short list of audited files, and CI fails when it
  appears anywhere else.
- The resident never writes the registry or Launch Services. Registration is
  the installer's and the settings window's.
- Every refusal the settings backend returns is a catalogue key, and a test
  fails when a key has no sentence in both languages.
- Four consecutive comment lines are prose and fail the build.

---

## Development Setup

### Requirements

- [Rust](https://rustup.rs) stable, with `rustfmt` and `clippy`
- [Node.js](https://nodejs.org) 22 or newer
- For Windows builds: Windows 10/11, Visual Studio 2022 (Desktop development
  with C++) and the WebView2 runtime, which ships with Windows 11
- For macOS builds: macOS 13 (Ventura) or newer with Xcode 15+

### Getting Started

```sh
git clone https://github.com/rgdevment/LinkUnbound.git
cd LinkUnbound
npm ci
npm ci --prefix app
```

### Running

The settings window bundles the resident as a sidecar, so the resident is
built first; without it the Tauri build stops at once.

```sh
bash scripts/sidecar.sh          # builds the resident into app/src-tauri/binaries/
cd app && npm run tauri dev      # the settings window, with hot reload
```

The resident on its own: `cargo run -p linkunbound-shell -- https://example.org`
opens the picker for that link; `cargo run -p linkunbound-shell --example preview`
opens it with sample rows and no resident (`--sheet`, `--light`, `--twelve`,
`--update`, `--stage=getting`, `--notice` show the other states).

Build a release with `npm run tauri build` from `app/`, never with
`cargo build` alone: the latter does not rebuild the front end and produces a
binary whose window opens empty. A debug settings binary is a console
application and dies with the console it was started from.

### Common Commands

| Command                                      | What it does                                   |
| :------------------------------------------- | :--------------------------------------------- |
| `cargo fmt --all`                            | Format the Rust sources                        |
| `cargo clippy --workspace --all-targets`     | Lint the Rust sources                          |
| `cargo test --workspace`                     | Run the Rust tests                             |
| `cargo deny check`                           | Audit licences and advisories                  |
| `npm run lint --prefix app`                  | Lint and format the front end                  |
| `npm test --prefix app`                      | Run the front-end tests                        |
| `npm run lint:md`                            | Lint the Markdown                              |
| `npm run notices`                            | Rewrite THIRD-PARTY-BUNDLED.md from the lockfiles |
| `cargo mutants -p <crate>`                   | Hunt for mutants the tests miss                |
| `npm run mutants --prefix app`               | The same, for the front end                    |

---

## Style Guide

- **Modern Rust** — edition 2024, and let the type system carry the invariants.
- **Descriptive names** — code should read like prose.
- **No comments** — write one only where a future reader would be surprised
  without it: a hidden constraint, a workaround, a subtle invariant. One line.
- **No comment prefixes** — no `TODO`, `NOTE`, `FIX`.
- **English everywhere** in code, identifiers and comments.
- **KISS** — keep it simple.

---

## Adding a Translation

The interface ships in English and Spanish, with automatic detection and an
override under **Application** in Settings. The words live in two catalogues,
one per side of the app:

- `crates/linkunbound-core/src/i18n.rs` — what the resident shows: the
  picker, the notice, the tray, the update strip. Each key carries its Spanish
  and English text side by side.
- `app/src/i18n.ts` — the settings window. One object per language with the
  same keys; a test fails when a key is missing from either.

To add a language, add its column to both catalogues, teach
`Language::of_tag` (Rust) and `spoken` (TypeScript) the locale tag, and open a
pull request. Neutral register, no regional variants: the Spanish is written
for every Spanish speaker at once.

---

## License and Rights

LinkUnbound is released under the **GNU General Public License v3.0
(GPL-3.0)** and offered under separate [commercial terms](COMMERCIAL.md) to
organisations that cannot comply with it. Your contributions are licensed the
same way.

### Contributor License Agreement

Because of that dual model, every contributor signs a one-time
[CLA](CLA.md) before their code can be merged. Offering commercial terms
requires the right to license the whole codebase that way, and that right has
to come from each author explicitly.

**You keep the copyright on your work.** The CLA is a licence you grant, not a
transfer of ownership.

The first time you open a Pull Request, a bot asks you to sign. Reply on that
Pull Request with exactly:

```text
I have read the CLA Document and I hereby sign the CLA
```

That is it — every later Pull Request from the same account is covered.

**Please leave tool co-authorship out of your commits.** Assistants are welcome
here — this project is built with them — but the credit line is for people. If
your editor adds a trailer naming one, drop it before you push. It changes
nothing about what you are allowed to submit; section 4 of the CLA already puts
the responsibility for generated code on you, whichever tool helped write it.

**In return, the project commits that:**

- The community edition stays available under the GPL-3.0.
- Your contribution is never removed from the open source project to make it
  exclusive to a commercial edition.
- Your authorship is preserved; history is not rewritten to erase it.
- No release already published is ever retroactively withdrawn.

Read [CLA.md](CLA.md) for the full text — it is short, and worth the two
minutes before you sign it.

If you would rather not sign, you can still use LinkUnbound, report bugs,
request features, discuss design, and fork the project under the GPL-3.0.
Only merging code into this repository requires the agreement.

---

## Questions?

- **GitHub Discussions** — For general questions and conversations.
- **Issues** — For specific bugs and suggestions.
- **Email** — [github@apirest.cl](mailto:github@apirest.cl) for sensitive matters.

There are no stupid questions. If you have doubts, ask.
