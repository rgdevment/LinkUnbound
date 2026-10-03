# Privacy Policy

**Last updated:** October 2, 2026

---

## The Short Version

**Everything stays on your computer.** LinkUnbound does not collect, transmit, or share any of your data. There is no cloud, no accounts, no telemetry, no analytics, no tracking. Nothing leaves your machine.

This is a technical fact you can verify yourself: the entire source code is [open and public](https://github.com/rgdevment/LinkUnbound). Read the code, run a network monitor, check for yourself.

---

## Privacy Philosophy

LinkUnbound was built with privacy as the foundation, not an afterthought. Every design decision starts from the same principle: **your data stays on your machine.**

- **Local-only by design** — Your data never leaves your computer.
- **No telemetry** — No measurement, no tracking, no analysis of your usage.
- **No analytics** — No Google Analytics, no App Insights, no Sentry, nothing.
- **No accounts** — No sign-up, no login, no user profiles.
- **No cloud sync** — Your configuration is yours alone.
- **No automatic reporting** — Nothing is sent anywhere without your explicit action.
- **Fully auditable** — Every line of code is open source under [GPLv3](LICENSE).

---

## What Data Does LinkUnbound Store?

LinkUnbound stores only what it needs to function:

### Browser List

| Data               | Purpose                           | Format |
| :----------------- | :-------------------------------- | :----- |
| Browser name       | Display in picker                 | JSON   |
| Executable path    | Launch the selected browser       | JSON   |
| Icon path          | Show browser icon in picker       | JSON   |
| Extra arguments    | Custom launch flags (if any)      | JSON   |
| Custom flag        | Distinguish manually-added browsers | JSON |

### Domain Rules

| Data            | Purpose                                             | Format |
| :-------------- | :-------------------------------------------------- | :----- |
| What it covers  | A site, a host or one exact address (e.g., `github.com`) | JSON |
| Browser ID      | Which browser opens that domain                     | JSON   |
| Originating app | Scope the rule to the app a link came from          | JSON   |
| Private flag    | Whether the rule opens the link in a private window | JSON   |

A rule for one exact address keeps that address whole, query string included, because that is
what it has to match. If a link carries a session or a reset token, choose the site or the host
instead of "this address" when you ask LinkUnbound to remember it.

### Originating Application

To offer "always open links from this app here", LinkUnbound has to know which
application asked the system to open the link. It is read like this:

| Platform | How the origin is determined                     | Reliability |
| :------- | :----------------------------------------------- | :---------- |
| Windows  | The application in the foreground at that moment | Approximate |
| macOS    | The application in the foreground at that moment | Approximate |

Neither system reports which application opened a link, so the foreground app stands in for it. LinkUnbound itself is never taken for the origin. On Windows the application is named by its executable (`slack`); on macOS a rule keeps its bundle identifier (`com.tinyspeck.slackmacgap`), which does not change with the system language, and the picker and Settings show its name. An app-scoped rule is only written when you tick "always open" on a link whose origin is known.

What this means in practice:

- **Only the application's name and identifier are read.** Not its windows, not its title, not
  its contents. The shell counts as an application like any other: a link
  opened from a file on the desktop reports `explorer`, and whether that is
  worth a rule is yours to decide, not ours. The picker names the rule that
  decided every time one does.
- **It is read at the moment a link arrives**, not continuously. LinkUnbound
  does not watch which applications you use.
- **It reaches the disk only if you ask it to.** The name is used in memory to
  match rules and to label the picker. It is written to `rules.json` only when
  you tick "always open" on a link whose origin is known.

### Links From Other Applications

An application can hand LinkUnbound a link through its own address,
`linkunbound://open?url=<link>`, even when LinkUnbound is not your default
browser. Such a link is treated exactly like one you clicked:

- **Nothing about it is kept** beyond what a click keeps — the address is held
  in memory while it is routed, like any other.
- **Nothing is sent back** to the application that sent it. It cannot learn
  which browser opened the link, whether a rule decided, or what your rules are.
- **Only web links are accepted.** A file, a script or anything else is dropped
  before it is shown or opened.

### No Log

LinkUnbound keeps no log of the links it handles. An address you open is held in memory while
it is routed and is written to disk only as part of a rule you asked it to remember.

### Extracted Icons

Browser icons are extracted locally from installed browser executables and stored as image files. These are visual assets only.

---

## Where Is Everything Stored?

All data is stored locally under your user profile.

**Windows** — `%LOCALAPPDATA%\LinkUnbound\`:

| Data     | Location                                    |
| :------- | :------------------------------------------ |
| Browsers | `%LOCALAPPDATA%\LinkUnbound\browsers.json`       |
| Rules    | `%LOCALAPPDATA%\LinkUnbound\rules.json`          |
| Settings | `%LOCALAPPDATA%\LinkUnbound\preferences.json`    |
| Last update check | `%LOCALAPPDATA%\LinkUnbound\update.json` |
| Icons    | `%LOCALAPPDATA%\LinkUnbound\icons\`              |

**macOS** — `~/Library/Application Support/LinkUnbound/`:

| Data     | Location                                                         |
| :------- | :--------------------------------------------------------------- |
| Browsers | `~/Library/Application Support/LinkUnbound/browsers.json`        |
| Rules    | `~/Library/Application Support/LinkUnbound/rules.json`           |
| Settings | `~/Library/Application Support/LinkUnbound/preferences.json`     |
| Last update check | `~/Library/Application Support/LinkUnbound/update.json`  |
| Icons    | `~/Library/Application Support/LinkUnbound/icons/`               |
| Previous default browser | `~/Library/Preferences/dev.rgdevment.linkunbound.plist` |

On macOS, setting LinkUnbound as the default browser records which application held that role before (its bundle identifier, nothing else), so that "stop being the default" can hand the links back to it.

`preferences.json` holds your settings — theme, language, shortcut — and two marks that exist only so the app can ask you for a GitHub star once, at a sensible moment rather than on day one: the date you first opened the settings window, and whether you have already answered. They are read on that screen and nowhere else, they never leave your machine, and deleting the file resets them.

These folders are protected by your operating system's user account permissions. Other users on the same computer cannot access them under normal conditions.

---

## What LinkUnbound Does NOT Do

- Does not send data to any server.
- Does not use cookies or tracking technologies.
- Does not create user accounts or profiles.
- Does not share data with third parties.
- Does not use advertising or ad networks.
- Does not monitor your browsing activity beyond processing each link.
- Does not phone home — except the update checker described below.

---

## Network Requests

LinkUnbound makes **one type of network request**:

### Update Checker

| Detail        | Value                                                                    |
| :------------ | :----------------------------------------------------------------------- |
| **Purpose**   | Check if a newer version of LinkUnbound is available                     |
| **URL**       | `https://raw.githubusercontent.com/rgdevment/LinkUnbound/manifest/release-manifest.json` |
| **Method**    | GET (read-only)                                                          |
| **Data sent** | Standard HTTP headers only — no user data                                |
| **Frequency** | At most once every 24 hours; asked for 20 seconds after the resident starts and every six hours after, and whenever the settings window opens |
| **Timeout**   | 5 seconds                                                                |
| **On failure**| Silent — the app continues working normally                              |

The resident that sits in the tray does not make the request itself: every six
hours it starts the settings binary with no window to ask on its behalf, and
that copy exits when it has written down the answer. The background check has a
switch under **Application** in Settings; off, the feed is only read when the
settings window is open.

**Important:**

- This request is **read-only** — it only downloads a small JSON response containing the latest version number. No data is ever uploaded.
- **No URLs, no rules, no browser information, no personal data** is ever sent.
- If an update is found, a strip appears atop the picker and a line under **About** in Settings. When you press Update, the installer is downloaded and run. Nothing is downloaded or installed without that press.
- The app works fully offline if the request fails or is blocked.

### Taking an update

Pressing Update downloads the installer from the releases page and runs it.
LinkUnbound makes that request itself; nothing is downloaded or installed
without the press. The installer is verified against a signature key built into
the copy you are running before a single byte of it is executed, and the address
it comes from has to be the release of the version you were offered. On a Mac
the installer asked for is the one built for its chip, so the name of that
download says whether the machine is Apple Silicon or Intel — as a download
made by hand from the releases page would.

A copy installed from the Microsoft Store takes its update from the Store
instead, and never reads the feed above. A copy installed with Homebrew updates
itself the same way as a downloaded one; `brew upgrade` works as well.

---

## Microsoft Store Distribution

LinkUnbound is available through the Microsoft Store. The Store version:

- **Follows the same privacy principles** as the standalone version.
- **Asks the Store about updates**, not the feed above: a release reaches the Store when its certification lets it through, which is not when it reaches everyone else.
- **Uses MSIX packaging** — installs and uninstalls cleanly with standard Windows mechanisms.
- **Microsoft Store policies** apply to distribution, but LinkUnbound itself does not share any data with Microsoft beyond what the Store platform requires for installation and updates.

For Microsoft's own privacy practices, refer to [Microsoft's Privacy Statement](https://privacy.microsoft.com/privacystatement).

---

## Data Deletion

### In-App

Settings → **Maintenance** tab provides:

- **Reset configuration** — clears all browsers, rules, and icons, then re-scans installed browsers.
- **Unregister** — removes LinkUnbound's browser registration from the system: the registry entries on Windows, `linkunbound://` included; on macOS, hands `http`, `https` and the web document types back to the application that held them before (Safari when none is remembered).

### Complete Removal

**Windows:**

1. Uninstall LinkUnbound (via Settings → Apps or the standalone uninstaller).
2. Delete the data folder: `%LOCALAPPDATA%\LinkUnbound\` (and the legacy `%APPDATA%\LinkUnbound\` if present from older versions)

**macOS:**

1. Drag `LinkUnbound.app` from `/Applications` to the Trash (or `brew uninstall --cask linkunbound`). The `linkunbound://` address goes with the app: macOS reads it from the app itself.
2. Delete the data folder: `~/Library/Application Support/LinkUnbound/`
3. Optional: remove preferences (`~/Library/Preferences/dev.rgdevment.linkunbound.plist`, and `com.rgdevment.linkunbound.plist` left by 1.x), saved app state under `~/Library/Saved Application State/`, and the login item under System Settings → General → Login Items if one was enabled.

After these steps, no LinkUnbound data remains on your system.

---

## Diagnostics Export

LinkUnbound can write a diagnostic report for troubleshooting: Settings → **About** → *Save report*. The file is written locally and **never sent anywhere** — you decide whether to share it and with whom.

It is a single Markdown file named `linkunbound-diagnostico.md`. On Windows it is written to your Desktop; where that folder does not exist, it goes to the system's temporary directory. The message on screen names the exact path it landed in.

### What the report contains

| Section | Content |
| :-- | :-- |
| Heading | The version of LinkUnbound you are running |
| `Sistema` | Operating system, whether LinkUnbound is registered and default, the health check, how many associations it holds, whether it starts with the system, whether Edge is installed, and the names of the browsers it detected |
| `Preferencias` | Theme, language, the global shortcut, and whether a rule announces itself when it decides |
| `Reglas` | Every rule you have: what it matches, the application it is tied to when it has one, the browser it opens in, and whether it opens privately |

### What the report does not contain

- **The addresses you open.** It carries your rules, not your browsing. A rule for one exact
  address is written as its scheme and host only (`https://mail.google.com/…`).
- **Passwords, tokens or anything from a URL's authority** — those never reach a file at all.
- **Icons, executables or the contents of any browser profile.**

**Worth knowing before you share one:** the rules section names the sites you
wrote rules for and the applications they answer to — `desde ms-teams`, for
example — and the system section lists the browsers installed on the machine.
That is the point of the file, because a rule is usually what decided where a
link went, but it is your information: open it and read it before attaching it
to a public issue.

---

## Children's Privacy

LinkUnbound does not collect personal information from anyone, including children under 13. The application has no accounts, no registration, and no data transmission.

---

## Open Source Transparency

The best privacy policy is one you can verify. LinkUnbound is **100% open source** under the [GNU General Public License v3.0](LICENSE):

- **Full source code:** [github.com/rgdevment/LinkUnbound](https://github.com/rgdevment/LinkUnbound)
- **Audit the code yourself** — every network request, every file write, every registry read.
- **Report concerns** — [open an issue](https://github.com/rgdevment/LinkUnbound/issues) or [email](mailto:github@apirest.cl).

See our [Security Policy](SECURITY.md) for responsible disclosure guidelines.

---

## Changes to This Policy

If this privacy policy changes, the changes will be:

- Committed to the public repository with a clear commit message.
- Reflected in the "Last updated" date above.
- Documented in the release notes.

Since LinkUnbound is open source, any change to privacy behavior would also be visible as a code change before it reaches you.

---

## Contact

- **Email:** [github@apirest.cl](mailto:github@apirest.cl)
- **GitHub Discussions:** [github.com/rgdevment/LinkUnbound/discussions](https://github.com/rgdevment/LinkUnbound/discussions)
- **Issues:** [github.com/rgdevment/LinkUnbound/issues](https://github.com/rgdevment/LinkUnbound/issues)
