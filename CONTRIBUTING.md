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
2. **Create a branch** from `main` (`git checkout -b feature/my-improvement`).
3. **Make your changes** following the style guide below.
4. **Run checks:**
   ```sh
   melos run format
   melos run analyze
   melos run test
   ```
5. **Open a Pull Request** to `main`.

### Translate

LinkUnbound supports English and Spanish. Help bring it to more languages — see the [localization section](#adding-a-translation) below.

### Improve Documentation

Found something confusing? Missing information? PRs welcome.

---

## Project Structure

LinkUnbound is a Dart/Flutter monorepo managed with [Melos](https://melos.invertase.dev/):

```
linkunbound_workspace/
  packages/
    core/               # Pure Dart — models, services, platform interfaces
      lib/src/
        models/         # Browser, BrowserConfig, Rule
        services/       # BrowserService, RuleService, LaunchService, etc.
        platform/       # Abstract platform interfaces
      test/             # Unit tests
  apps/
    linkunbound/        # Flutter app — UI, platform implementations, providers
      lib/
        platform/       # Windows and macOS-specific implementations
        ui/             # Picker, Settings, shared widgets
        providers.dart  # Riverpod providers
        main.dart       # Entry point
```

**Key conventions:**

- Business logic lives in `packages/core` (pure Dart, no Flutter dependency).
- UI and platform implementations live in `apps/linkunbound`.
- State management uses Riverpod with `NotifierProvider` pattern.
- Tests go in `packages/core/test/`.

---

## Development Setup

### Requirements

- [Flutter SDK](https://docs.flutter.dev/get-started/install) (stable channel)
- [Melos](https://melos.invertase.dev/) (`dart pub global activate melos`)
- For Windows builds: Windows 10/11 with Visual Studio 2022 (Desktop development with C++)
- For macOS builds: macOS 13 (Ventura) or newer with Xcode 15+ (open `apps/linkunbound/macos/Runner.xcworkspace` to edit Swift sources, native channels, signing or entitlements)

### Getting Started

```sh
git clone https://github.com/rgdevment/LinkUnbound.git
cd LinkUnbound
melos bootstrap
```

### Running

```sh
cd apps/linkunbound
flutter run -d windows   # on Windows
flutter run -d macos     # on macOS
```

### Common Commands

| Command              | What it does                     |
| :------------------- | :------------------------------- |
| `melos run format`   | Format all packages              |
| `melos run analyze`  | Run `dart analyze` everywhere    |
| `melos run test`     | Run tests in all packages        |
| `melos bootstrap`    | Install dependencies + link      |

---

## Style Guide

- **Modern Dart** — Use latest language features.
- **Descriptive names** — Code should read like prose.
- **Minimal comments** — Only when clarifying non-obvious logic. Comments in English.
- **No comment prefixes** — No `TODO`, `NOTE`, `FIX`, etc.
- **KISS** — Keep it simple.

---

## Adding a Translation

LinkUnbound uses Flutter's standard ARB-based localization.

1. Copy `apps/linkunbound/lib/l10n/app_en.arb` as your base.
2. Name your file with the language code: `app_de.arb`, `app_fr.arb`, etc.
3. Translate the values (keep the keys in English).
4. Run `flutter gen-l10n` to regenerate localization classes.
5. Test by changing the language in Settings.
6. Submit a Pull Request.

**Guidelines:**

- Keep translations concise — UI space is limited.
- Use formal or neutral tone.
- Preserve placeholders like `{name}` or `{count}`.
- Don't translate brand names (LinkUnbound, Windows, etc.).

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
