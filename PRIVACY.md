# Privacy Policy

**Last updated:** August 4, 2026

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
| Domain          | Match URLs to a browser (e.g., `github.com`)        | JSON   |
| Browser ID      | Which browser opens that domain                     | JSON   |
| Originating app | Scope the rule to the app a link came from          | JSON   |
| Private flag    | Whether the rule opens the link in a private window | JSON   |

### Originating Application

To offer "always open links from this app here", LinkUnbound has to know which
application asked the system to open the link. It is read like this:

| Platform | How the origin is determined                     | Reliability |
| :------- | :----------------------------------------------- | :---------- |
| Windows  | Parent process of the process the shell launched | Accurate    |
| macOS    | The application in the foreground at that moment | Approximate |

macOS does not report which application opened a link, so the foreground app stands in for it. An app-scoped rule is only written when you tick "always open" on a link whose origin is known.

What this means in practice:

- **Only the application name is read.** Not its windows, not its title, not
  its contents. `explorer`, `cmd` and `powershell` are discarded, since they
  are the shell itself rather than an app worth writing a rule about.
- **It is read at the moment a link arrives**, not continuously. LinkUnbound
  does not watch which applications you use.
- **It reaches the disk only if you ask it to.** The name is used in memory to
  match rules and to label the picker. It is written to `rules.json` only when
  you tick "always open" on a link whose origin is known.

### Navigation Log

| Data      | Purpose                                      | Format     |
| :-------- | :------------------------------------------- | :--------- |
| Timestamp | When a link was processed                    | Plain text |
| Log level | Severity (INFO, WARNING, etc.)               | Plain text |
| Message   | Application events and errors                | Plain text |

The navigation log **does not contain actual URLs**. All URLs are automatically redacted at write time before reaching the log file — they are replaced with privacy-safe placeholders like `https://<redacted>/2 segments`. The original URLs exist only in memory during processing and are never persisted to disk.

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
| Log      | `%LOCALAPPDATA%\LinkUnbound\navigate.log`        |
| Crash log | `%LOCALAPPDATA%\LinkUnbound\startup_crash.log`  |
| Icons    | `%LOCALAPPDATA%\LinkUnbound\icons\`              |

**macOS** — `~/Library/Application Support/LinkUnbound/`:

| Data     | Location                                                         |
| :------- | :--------------------------------------------------------------- |
| Browsers | `~/Library/Application Support/LinkUnbound/browsers.json`        |
| Rules    | `~/Library/Application Support/LinkUnbound/rules.json`           |
| Log      | `~/Library/Application Support/LinkUnbound/navigate.log`         |
| Crash log | `~/Library/Application Support/LinkUnbound/startup_crash.log`   |
| Icons    | `~/Library/Application Support/LinkUnbound/icons/`               |

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
| **URL**       | `https://api.github.com/repos/rgdevment/LinkUnbound/releases/latest`     |
| **Method**    | GET (read-only)                                                          |
| **Data sent** | Standard HTTP headers only — no user data                                |
| **Frequency** | At most once every 6 hours                                               |
| **Timeout**   | 5 seconds                                                                |
| **On failure**| Silent — the app continues working normally                              |

The resident process is long-lived, so the check happens on that interval
rather than only once at launch.

**Important:**

- This request is **read-only** — it only downloads a small JSON response containing the latest version number. No data is ever uploaded.
- **No URLs, no rules, no browser information, no personal data** is ever sent.
- If an update is found, a non-invasive indicator appears in the app. No automatic download or installation occurs.
- The app works fully offline if the request fails or is blocked.

### User-Initiated Navigation

When you click "Download" on an update notification, LinkUnbound opens the GitHub release page in your default browser. This is a standard browser navigation initiated by your action — LinkUnbound does not make this request itself.

---

## Microsoft Store Distribution

LinkUnbound is available through the Microsoft Store. The Store version:

- **Follows the same privacy principles** as the standalone version.
- **Makes the same single read-only request** to check for updates via the GitHub Releases API.
- **Uses MSIX packaging** — installs and uninstalls cleanly with standard Windows mechanisms.
- **Microsoft Store policies** apply to distribution, but LinkUnbound itself does not share any data with Microsoft beyond what the Store platform requires for installation and updates.

For Microsoft's own privacy practices, refer to [Microsoft's Privacy Statement](https://privacy.microsoft.com/privacystatement).

---

## Data Deletion

### In-App

Settings → **Maintenance** tab provides:

- **Reset configuration** — clears all browsers, rules, and icons, then re-scans installed browsers.
- **Unregister** — removes LinkUnbound's browser registration from the system: the registry entries on Windows, the Launch Services association on macOS.

### Complete Removal

**Windows:**

1. Uninstall LinkUnbound (via Settings → Apps or the standalone uninstaller).
2. Delete the data folder: `%LOCALAPPDATA%\LinkUnbound\` (and the legacy `%APPDATA%\LinkUnbound\` if present from older versions)

**macOS:**

1. Drag `LinkUnbound.app` from `/Applications` to the Trash (or `brew uninstall --cask linkunbound`).
2. Delete the data folder: `~/Library/Application Support/LinkUnbound/`
3. Optional: remove preferences (`~/Library/Preferences/com.rgdevment.linkunbound.plist`) and saved app state (`~/Library/Saved Application State/com.rgdevment.linkunbound.savedState/`).

After these steps, no LinkUnbound data remains on your system.

---

## Diagnostics Export

LinkUnbound includes an optional **Export diagnostics** feature (Settings → Maintenance) that generates a ZIP file for troubleshooting. This file is created locally and **never sent automatically** — you choose whether and where to share it.

### What the ZIP Contains

| File                  | Content                                                                   |
| :-------------------- | :------------------------------------------------------------------------ |
| `system_info.txt`     | OS version, locale, app version, executable path, data files              |
| `registry.txt`        | Windows only — LinkUnbound's own registry entries, those three keys alone |
| `launch_services.txt` | macOS only — the system's URL-handler associations (see the note below)   |
| `navigate.log`        | Last 200 lines of the navigation log (URLs already redacted)              |

### What the ZIP Does NOT Contain

- **Browser list** (`browsers.json`) — not included
- **Domain rules** (`rules.json`) — not included, so no originating app names leave your machine through the export
- **Icons** — not included
- **Actual URLs** — URLs are redacted at the source (log writing), not at export time

**Worth knowing before you share one:** on macOS the export includes
`launch_services.txt`, which lists which application your system uses to open
each URL scheme and file type. That is information about your machine rather
than about your browsing, but it does name other software you have installed.
Open the ZIP and look before attaching it to a public issue.

### URL Redaction

URLs are redacted **at write time** — before they ever reach the log file on disk. Every URL is replaced with a privacy-safe placeholder that preserves only the protocol and the number of path segments:

- `https://mail.google.com/inbox/123` → `https://<redacted>/3 segments`
- `http://internal.company.net/app` → `http://<redacted>/2 segments`

This means the `navigate.log` file on your machine never contains real URLs. The diagnostics export simply copies the last 200 lines of this already-redacted log.

Redaction covers the whole log record, including attached error objects and stack traces. This matters because a failed browser launch raises an error whose text embeds the full command line — that is, the URL. The same redaction is applied to `startup_crash.log`, a separate file written only when the app fails during startup; it is capped in size and is safe to delete at any time.

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
