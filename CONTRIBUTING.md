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

LinkUnbound is a Cargo workspace with a Tauri front end:

```text
LinkUnbound/
  crates/
    linkunbound-core/     # Pure Rust — models, rules, URL handling
      src/
        browser.rs        # Browser and Profile
        rule.rs           # HostPattern, Rule, RuleSet
        lib.rs            # URL helpers
  app/
    src/                  # The picker and settings, in React
    src-tauri/            # The desktop shell: windows, tray, single instance
      src/
      capabilities/       # What the front end is allowed to call
```

**Key conventions:**

- Business logic lives in `crates/linkunbound-core`. It depends on neither
  Tauri nor any platform API, and CI enforces that.
- The core never prints. The picker would inherit the output as garbage.
- `unsafe` is forbidden workspace-wide. A platform crate that genuinely needs
  it declares `allow(unsafe_code)` in one audited place, and CI fails when it
  appears anywhere else.
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
npm install --prefix app
```

### Running

```sh
cd app
npm run tauri dev
```

Build with `cargo tauri build`, never with `cargo build` alone: the latter does
not rebuild the front end and produces a binary whose window opens empty.

### Common Commands

| Command                        | What it does                        |
| :----------------------------- | :---------------------------------- |
| `cargo fmt --all`              | Format the Rust sources             |
| `cargo clippy --workspace`     | Lint the Rust sources               |
| `cargo test --workspace`       | Run the Rust tests                  |
| `cargo deny check`             | Audit licences and advisories       |
| `npm run lint --prefix app`    | Lint and format the front end       |
| `npm test --prefix app`        | Run the front-end tests             |

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

The 2.0 series is being rebuilt and its localisation is not in place yet, so
the interface currently ships in English only. Translations are welcome once
that lands; until then, open an issue if you want to be told when it does.

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
