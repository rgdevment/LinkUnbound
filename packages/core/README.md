# linkunbound_core

Platform-agnostic core of [LinkUnbound](../../README.md): the models, services
and platform contracts that decide **which browser opens a link**, with no
Flutter and no `dart:ui` dependency.

Everything here is pure Dart so it can be unit tested without a window, an
operating system handler, or a running app. The Flutter application and the
per-OS implementations live in [`apps/linkunbound`](../../apps/linkunbound).

## What lives here

| Area | Contents |
| :--- | :--- |
| `models/` | `Browser`, `BrowserConfig`, `Rule` |
| `services/` | Browser detection and persistence, rule matching, launching, logging, update checks |
| `platform/` | Abstract contracts each OS implements: registration, startup, inbound events, diagnostics |
| `url_utils.dart` | URL unwrapping, redaction and the launchability gate |
| `private_mode.dart` | The private-window switch each browser family expects |

## Two rules that shape this package

**No Flutter imports.** If something needs a `BuildContext`, a channel or a
window, it belongs in the app, not here. This is what keeps the test suite fast
and the logic verifiable in isolation.

**The URL is untrusted.** Inbound events arrive over IPC from any local
process, so `isLaunchableUrl` is the single gate every URL passes before
reaching `Process.start`. Anything that starts with `-` is a browser switch,
not a link.

## Tests

```sh
cd packages/core
dart test
```

## Licence

GPL-3.0, with commercial terms available — see [LICENSE](../../LICENSE) and
[COMMERCIAL.md](../../COMMERCIAL.md).
