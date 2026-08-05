# LinkUnbound (app)

Flutter app for LinkUnbound — UI, platform implementations, and entry point. Business logic lives in `packages/core`.

For the full project description, philosophy, installation, and architecture, see the [root README](../../README.md).

## Run locally

```sh
cd apps/linkunbound
flutter run -d windows   # Windows 10/11 + Visual Studio 2022 (Desktop C++)
flutter run -d macos     # macOS 13+ + Xcode 15+
```

## Tests and analysis

```sh
flutter test                                # widget/unit tests for the app layer
dart analyze --fatal-infos                  # zero issues
flutter test --coverage                     # generates coverage/lcov.info
```

## Layout

```
lib/
  main.dart                # entry point
  app.dart                 # MaterialApp + router
  bootstrap.dart           # platform-agnostic startup (single-instance, tray, IPC)
  providers.dart           # Riverpod providers
  l10n/                    # ARB-based localization (en, es)
  platform/
    platform_bindings.dart # abstract contract per OS
    windows/               # named pipe + registry + tray (Win32)
    macos/                 # Apple Events + Launch Services + LSUIElement
  ui/
    picker/                # floating browser picker
    settings/              # General, Rules, Maintenance, About tabs
    shared/                # theme + reusable widgets
test/                      # widget + unit tests for the app layer
```

## Platform notes

**Windows.** Default-browser registration writes the app's own ProgId, `RegisteredApplications` and `StartMenuInternet` keys, reconciled on every launch; `UserChoice` stays under Windows' control. Single-instance + IPC via a named pipe (`\\.\pipe\LinkUnbound`). Tray icon via `tray_manager`. Native packaging in `windows/packaging/{exe,msix}`.

**macOS.** Default-browser registration via `NSWorkspace.setDefaultApplication`. URL events delivered through `application:openURLs:` and forwarded to Dart via `MethodChannel`. The app runs as `LSUIElement` (menu bar only, no Dock icon). Native sources live in `macos/Runner/`. Distribution is signed/notarized via `scripts/macos/release.sh` and shipped through the `rgdevment/tap` Homebrew cask.

## Edit native code

- Windows: open `windows/runner/` in Visual Studio (or VS Code with C++ extension) — uses CMake.
- macOS: open `macos/Runner.xcworkspace` in Xcode 15+ for Swift sources, signing, entitlements, and Info.plist.
