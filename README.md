<div align="center">

<img src="app/src-tauri/icons/icon.png" alt="LinkUnbound icon" width="96" height="96"/>

# LinkUnbound

**A free, open source browser picker for Windows and macOS. Choose which browser opens every link.**

**Open source. Local-first. Privacy-first. Zero telemetry.**

<p>
  <a href="https://github.com/rgdevment/LinkUnbound/releases">
    <img src="https://img.shields.io/github/v/release/rgdevment/LinkUnbound?include_prereleases&style=flat-square&label=Latest&color=0078D4" alt="Latest Release"/>
  </a>
  <img src="https://img.shields.io/badge/Platform-Windows-0078D4?style=flat-square" alt="Platform: Windows"/>
  <img src="https://img.shields.io/badge/Platform-macOS-000000?style=flat-square&logo=apple" alt="Platform: macOS"/>
  <a href="#license">
    <img src="https://img.shields.io/github/license/rgdevment/LinkUnbound?style=flat-square&color=lightgrey" alt="License GPL-3.0"/>
  </a>
  <a href="https://github.com/sponsors/rgdevment">
    <img src="https://img.shields.io/badge/GitHub-Sponsor-EA4AAA?style=flat-square&logo=githubsponsors&logoColor=white" alt="Sponsor on GitHub"/>
  </a>
  <a href="https://buymeacoffee.com/rgdevment">
    <img src="https://img.shields.io/badge/Buy%20Me%20a%20Coffee-☕-FFDD00?style=flat-square&logo=buy-me-a-coffee&logoColor=black" alt="Buy Me a Coffee"/>
  </a>
</p>

<p>
  <img src="resources/assets/github-en/a-dark.png" alt="LinkUnbound: choose which browser opens every link. The picker sits next to the link, listing Google Chrome, Mozilla Firefox and Vivaldi with the profiles each one carries" width="92%"/>
</p>

<h4>Download LinkUnbound</h4>

<p align="center">
  <a href="https://apps.microsoft.com/detail/9N9F7C8Q43KC">
    <img src="https://img.shields.io/badge/Windows-Microsoft_Store-0078D4?style=for-the-badge&logo=microsoft" alt="Get LinkUnbound browser picker from Microsoft Store"/>
  </a>
  &nbsp;
  <a href="#getting-started">
    <img src="https://img.shields.io/badge/macOS-Homebrew-FBB040?style=for-the-badge&logo=homebrew&logoColor=black" alt="Install LinkUnbound browser picker via Homebrew on macOS"/>
  </a>
</p>

<p align="center">
  <sub>Prefer a direct download? <a href="https://github.com/rgdevment/LinkUnbound/releases/latest">GitHub Releases</a> has standalone installers — Windows (.exe) · macOS (.dmg, one per chip)</sub>
</p>

</div>

---

**LinkUnbound** is a free, open source **browser picker** for Windows and macOS — a small tool that makes itself the default browser and then asks *you* which real browser should open each link. Click a link in Teams, Outlook, Slack, Discord, a PDF, a terminal, anywhere: if you have a rule for it, the right browser opens at once; if not, a little **browser chooser** pops up next to your cursor and you pick with a click or a key.

It is the missing piece for anyone who lives with more than one browser: work in one, personal life in another, a client's tools in a third profile. Instead of changing the system default every week, you set a **rule per site, per subdomain, per exact address or per app** — "everything from Slack in Brave", "docs.google.com in the work profile of Chrome" — and the rest goes through the picker.

There is no company behind this. I am one developer who got tired of the operating system deciding which browser opens a link, built this for myself, and put it out for whoever has the same itch. That shapes everything about it: **no ads, no telemetry, no analytics, no accounts, no subscriptions, no cloud, no data collection** — the only thing it ever asks the network is whether a new version exists, and you can turn that off. Your rules live in a small JSON file on your own disk, and you can read every line of the code that touches them.

---

## Table of Contents

- [What It Does](#what-it-does)
- [Privacy and Security](#privacy-and-security)
- [Getting Started](#getting-started)
- [How It Works](#how-it-works)
- [Domain Rules](#domain-rules)
- [Architecture](#architecture)
- [FAQ](#faq)
- [Localization](#localization)
- [Want to Help?](#want-to-help)
- [Support the Project](#support-the-project)
- [Other Tools](#other-tools)
- [Alternatives](#alternatives)
- [License](#license)

---

## What It Does

<p align="center">
  <img src="resources/assets/linkUnbound_2_picker.gif" alt="A link is clicked inside a desktop application; the picker opens beside the cursor naming the address and the application it came from, and the link opens in the browser chosen" width="700"/>
</p>

The picker names the address and the application the link came from, and each browser carries the profile it will open in. The settings window covers the rest: the rules and how they are matched, the browsers it found and the ones you add by hand, the diagnostic report, and what is kept on disk.

- **Registers as default browser** — intercepts every link click system-wide on Windows and macOS
- **Shows a floating picker** near your cursor, in two looks: a classic list or a mosaic of tiles. Pick with the mouse or the keys `1`–`9`; hold **Shift** for a private window; `Ctrl+C` copies the address; `Esc` puts it away
- **Remembers the choice at the reach you pick** — this URL, this subdomain, the whole site, or everything a given app sends
- **Rules from Settings too** — a rule for a site you have not visited yet, or for an app, without waiting for the picker
- **Unwraps Microsoft SafeLinks** and the Edge-only scheme Teams and Outlook wrap links in, so rules see the real destination
- **Tells you when a rule decided** — a small notice with an Undo, six seconds, no focus taken
- **Opens local documents** when you choose it for them — `.html`, `.pdf` and the rest on Windows, `.html` and `.xhtml` on macOS
- **Runs silently** in the system tray (or menu bar on macOS) — starts at sign-in, stays out of the way
- **Detects installed browsers** and their Chromium profiles — or add custom ones manually
- **Opens in a private window** using the switch each browser family expects
- **Updates itself** — looks at the release feed in the background, says so in the picker and in Settings, and one press installs it; a Microsoft Store copy updates through the Store
- **Tells you when links are not arriving** — Settings explains why and offers the repair
- **Two languages, three themes** — English and Spanish with automatic detection; light, dark or the system's

---

## Privacy and Security

**Everything stays local.** LinkUnbound is built on a single, non-negotiable principle: your data never leaves your computer.

- **Local-only storage** — browser list, domain rules, and logs stay on your machine
- **URL redaction** — URLs are automatically redacted in logs at write time; the log file never contains actual URLs
- **No tracking** — no telemetry, no analytics, no hidden collection
- **No accounts** — no sign-up, no login, no profiles
- **One network request** — a read-only check of the release feed the project publishes on GitHub for updates (no user data sent, works offline)

**By design, LinkUnbound will never have:** accounts, subscriptions, ads, cloud sync, or data collection of any kind.

For responsible disclosure and security contact info, see [SECURITY.md](SECURITY.md). Full privacy details in [PRIVACY.md](PRIVACY.md).

---

## Getting Started

### Requirements

- Windows 10 or 11, **or** macOS 13 (Ventura) or newer
- At least two browsers installed

### Installation

**Windows — Microsoft Store** (recommended): [Get it from Microsoft Store](https://apps.microsoft.com/detail/9N9F7C8Q43KC) — one click, auto-updates, no security warnings.

**Windows — standalone installer**: download from [GitHub Releases](https://github.com/rgdevment/LinkUnbound/releases/latest).

**macOS** — install via Homebrew (signed and notarized):

```bash
brew tap rgdevment/tap
brew install --cask linkunbound          # stable
brew install --cask linkunbound-beta     # pre-release
```

Or download the `.dmg` directly from [GitHub Releases](https://github.com/rgdevment/LinkUnbound/releases/latest). There are two: `aarch64` for Apple Silicon and `x86_64` for Intel — Apple menu › *About This Mac* says which one yours is. Homebrew picks by itself.

<details>
<summary><strong>Windows standalone: security warnings</strong></summary>

Since LinkUnbound is an independent open source project, the installer is signed with a self-signed certificate. Windows and your browser may show security warnings — this is normal and expected.

- **Browser:** Chrome/Edge may block the download — click Keep or Keep anyway.
- **SmartScreen:** Click More info → Run anyway (only happens once).
- **Why?** Code signing certificates cost $200–800/year. The code is 100% open source — you can inspect every line.

</details>

### Setup

**Windows:**

1. Run the installer; it registers LinkUnbound as a browser, starts it, and offers to remove a 1.x install if it finds one
2. In the settings window, under **Links**, click **Open Settings** beside *Choose LinkUnbound in Windows* — Windows Settings opens on LinkUnbound; make it the default
3. Done — every link now goes through LinkUnbound. The same page lets you pick it for `.html`, `.pdf` and other documents

**macOS:**

1. Launch **LinkUnbound** from Applications (or Spotlight); the settings window opens
2. In **Links**, click **Set** under *Set as the default browser*
3. macOS asks you to confirm the change → **Use "LinkUnbound"**
4. Done — every link now goes through LinkUnbound. Local `.html` and `.xhtml` files go through it too; `.svg` only if you choose it for them

---

## How It Works

**Link click with a rule:** the browser opens at once, and a small notice in the corner says which rule decided, with an **Undo** for six seconds. The notice never takes the keyboard.

**Link click without a rule:** a picker appears near your cursor, showing where the link comes from and the app it was clicked in. Pick a browser with the mouse or `1`–`9`. Hold **Shift** (or pin it with the *Private* pill) to open a private window. To remember the choice, pick a reach first — *this URL*, *this subdomain*, *the whole site* or *from this app* — with the arrows or the pills, then pick the browser. `Ctrl+C` copies the address; `Esc` puts the picker away. Links clicked while it is open wait their turn.

**Two looks.** *Classic* is a list; *Mosaic* is a sheet of tiles that shows every profile at a glance. Choose under **Application** in Settings.

<p align="center">
  <img src="resources/assets/linkUnbound_2_0_picker_en.png" alt="The classic picker: a list of browsers with keys 1 to 3, the link's site and the app it came from"/>
  &nbsp;&nbsp;
  <img src="resources/assets/linkUnbound_2_0_picker_mosaic_en.png" alt="The mosaic picker: tiles for every browser and profile, six of them here"/>
</p>

**Settings (tray):** double-click the tray icon or right-click → Settings. Six pages:

- **Links** — whether the system sends links here, what is wrong when it does not and the button that fixes it; on Windows, the associations held
- **Rules** — every rule, the most precise one deciding; change where each opens, add one, remove one
- **Browsers** — what was detected, with profiles; hide, duplicate, or add a custom one with its arguments, and look for browsers again after installing one
- **Application** — theme, language, picker look, start at sign-in, the global shortcut, background update checks
- **Maintenance** — the two that destroy something: reset the configuration, and remove LinkUnbound from the system's list
- **About** — version, licence, the update button, the candidate-versions switch, the diagnostic report when something goes wrong, and support links

**Updates.** Every six hours the resident asks the release feed, without a window. A newer version shows up as a strip atop the picker and in About; **Update** downloads the signed installer and runs it, and the resident comes back on its own. A copy from the Microsoft Store updates through the Store; a Homebrew copy updates itself the same way as a downloaded one (the cask says `auto_updates`), and `brew upgrade` works as well. Turn the background check off under Application if you prefer to ask by hand.

---

## Domain Rules

A rule covers one of four reaches: an exact address, a host (`mail.google.com`), a whole site (`google.com`, which covers `mail.google.com`, `drive.google.com` and the rest), or everything a given application sends. The narrowest reach that matches wins, and a rule scoped to an app wins over any domain rule — naming the origin is a deliberate statement, and a generic domain rule should not override it silently. Between two rules of the same reach, the one higher in the Rules page answers.

Rules are written from the picker — pick the reach, then the browser — or from the **Rules** page, where the same rule is typed in. A rule can also ask for a private window, if the browser has one.

---

## Architecture

Two binaries, one package: a resident that receives the links and draws the picker, and a settings window opened on demand. How they talk to each other on each system is in [CONTRIBUTING.md](CONTRIBUTING.md#architecture).

**Coming from 1.x.** 2.0 is not compatible with the settings of the 1.x line, and does not promise to carry them over: it starts from what it can read and keeps the rest out of the way. On Windows it reads the rules and browsers of a standalone 1.x install where they are, and keeps a copy of the originals as `rules.1x.json` and `browsers.1x.json` before writing its own format; a copy installed from the Microsoft Store kept its files inside its package and they are not read. Going back to 1.x means restoring those copies over `rules.json` and `browsers.json`. Remove 1.x before installing 2.0 — the installer offers to — or the two fight over the same registration; a Store copy of 1.x has to be uninstalled by hand. On macOS 2.0 ships under the bundle identifier `dev.rgdevment.linkunbound`; 1.x was `com.rgdevment.linkunbound`, kept its files under that name, and nothing of it is read. macOS treats them as two applications: quit 1.x, remove it, then open Settings and set LinkUnbound as the default once more, and remove the old entry under System Settings → General → Login Items if one is left behind.

---

## FAQ

**Is LinkUnbound free?**
Yes. Completely free and open source. No premium tiers, no subscriptions, no paywalls — ever. That covers using it anywhere, including across a company. Only redistributing it inside a product of your own needs [separate terms](COMMERCIAL.md).

**Does it track my browsing?**
No. LinkUnbound does not track or transmit anything. URLs are processed in memory and automatically redacted before being written to the navigation log — the log file never contains actual URLs, only privacy-safe placeholders.

**Does it need internet?**
No. LinkUnbound works fully offline. The only network request is a lightweight update check against the release feed the project publishes on GitHub — no user data sent. The app works perfectly without a connection.

**Where is my data stored?**
Everything stays on your machine — `%LOCALAPPDATA%\LinkUnbound\` on Windows, `~/Library/Application Support/LinkUnbound/` on macOS. Browser list (`browsers.json`), rules (`rules.json`), preferences (`preferences.json`), what the last update check found (`update.json`), the navigation log (`navigate.log`), and extracted icons. Removing the program keeps the rules and browsers unless you ask otherwise.

**Does it work with any browser?**
Yes. LinkUnbound detects all browsers registered with the operating system. You can also add custom browsers manually with any executable path and arguments.

**Can I use it with Microsoft SafeLinks?**
Yes. LinkUnbound unwraps SafeLinks before matching rules, so your rules work on the actual destination — and the picker names it, not the wrapper.

**Why do Teams and Outlook still open Edge?**
Both carry a setting of their own that sends links to whatever your default browser is, and that is the first thing to change: in Teams it lives under *Settings → Files and links*, and in classic Outlook under *File → Options → Advanced → «Open hyperlinks from Outlook in»*. Microsoft moves these menus about, so look for «link open preference» or «hyperlinks» if they are not where this says. Set that way, their links reach LinkUnbound like anybody else's.

What is left after that is a channel of Microsoft's own that hands links straight to Edge, skipping the default browser altogether, and Windows 11 no longer lets another application stand in the middle of it. When one of those arrives wrapped in a SafeLink it is unwrapped; the rest is Microsoft's to change.

**I do not see the tray icon on Windows.**
It is there, in the hidden icons behind the `^` at the right of the taskbar: Windows keeps every new application's icon in that overflow until you drag it onto the bar, or promote it under Settings → Personalization → Taskbar → Other system tray icons. Closing the settings window never stops the resident; only *Exit* in the tray menu does.

**Does the picker have a keyboard?**
Yes: `1`–`9` open a row, `Shift` holds a private window, `Ctrl+P` pins it, the arrows or `Tab` walk the reaches, `Ctrl+C` copies the address and `Esc` closes. A global shortcut (default `Alt+Shift+L`) opens Settings.

---

## Localization

LinkUnbound supports English and Spanish with automatic language detection. You can override the language in Settings.

| Language | Tag   | Status   |
| :------- | :---: | :------: |
| English  | en    | Complete |
| Spanish  | es    | Complete |

Want to add your language? See the [translation guide](CONTRIBUTING.md#adding-a-translation) in CONTRIBUTING.md.

---

## Want to Help?

Contributions are always appreciated:

- **Write Code** — Fix bugs or add features. See [CONTRIBUTING.md](CONTRIBUTING.md).
- **Translate** — Add your language.
- **Report Bugs** — [Open an issue](https://github.com/rgdevment/LinkUnbound/issues/new).
- **Share Ideas** — Tell me what you wish this browser picker could do.

---

## Support the Project

LinkUnbound is free and will always be free. No ads, no premium tiers, no paywalls. If it saves you time and you want to support continued development, sponsor it on GitHub or buy me a coffee:

<p align="center">
  <a href="https://github.com/sponsors/rgdevment">
    <img src="https://img.shields.io/badge/GitHub-Sponsor-EA4AAA?style=for-the-badge&logo=githubsponsors&logoColor=white" alt="Sponsor on GitHub"/>
  </a>
  &nbsp;
  <a href="https://buymeacoffee.com/rgdevment">
    <img src="https://img.shields.io/badge/Buy%20Me%20a%20Coffee-%E2%98%95-FFDD00?style=for-the-badge&logo=buy-me-a-coffee&logoColor=black" alt="Buy Me a Coffee"/>
  </a>
</p>

Every contribution helps keep these tools alive and maintained. But if you can't or don't want to donate — that's completely fine. Star the repo, share it with someone, or just use it. That's enough.

---

## Other Tools

I build free, open source tools focused on privacy and productivity. If you like LinkUnbound, you might also find these useful:

- **[CopyPaste](https://github.com/rgdevment/CopyPaste)** — A local-first clipboard manager and clipboard history tool for Windows, macOS, and Linux. Same philosophy: no ads, no telemetry, no accounts. Everything local.

---

## Alternatives

Other browser pickers, so you can pick the one that fits. Platform and licence checked against each project in September 2026; everything else changes, so go and look.

| Project | Platform | Licence |
| :-- | :-- | :-- |
| [Finicky](https://github.com/johnste/finicky) | macOS | MIT |
| [Browserino](https://github.com/AlexStrNik/Browserino) | macOS | GPL-3.0 |
| [Browserosaurus](https://github.com/will-stone/browserosaurus) | macOS | GPL-3.0, no longer maintained |
| [BrowserPicker](https://github.com/mortenn/BrowserPicker) | Windows | MIT |
| [Browser Tamer](https://github.com/aloneguid/bt) | Windows, Linux | Apache-2.0 |
| [Junction](https://github.com/sonnyp/Junction) | Linux | GPL-3.0 |
| [Switchbar](https://switchbar.app/) | Windows, macOS | Freemium, closed source |
| [Choosy](https://choosy.app/) | macOS | Paid |
| [Velja](https://sindresorhus.com/velja) | macOS | Free, closed source |

What LinkUnbound does that most of these do not: **one build for Windows and macOS**, rules that match **the app a link came from** and not only its address, **browser profiles** detected on their own, and signed automatic updates. Finicky is configured in a file and is macOS only; Junction is Linux only; Browserosaurus now points its own readers at Browserino, which is macOS only too.

---

## License

**LinkUnbound** — A free, open source browser picker for Windows and macOS.
Copyright (C) 2026 Mario Hidalgo G. (rgdevment)

This program comes with ABSOLUTELY NO WARRANTY.
This is free software, and you are welcome to redistribute it under certain conditions.
Distributed under the **GNU General Public License v3.0**. See [LICENSE](LICENSE) for details.

LinkUnbound is dual licensed. The GPL-3.0 covers everyone using, deploying,
auditing or forking it — which is almost everybody, and it costs nothing.
Redistributing it, or shipping it inside a product you distribute, needs
separate terms: see [COMMERCIAL.md](COMMERCIAL.md).

Contributions require a one-time [CLA](CLA.md); you keep the copyright on your
work.

What ships inside the binaries, each under its own licence, is listed in
[THIRD-PARTY-BUNDLED.md](THIRD-PARTY-BUNDLED.md); `npm run notices` writes it
from the lockfiles.

---

I built LinkUnbound because I was tired of my OS not letting me choose which browser opens a link. This is a personal tool, built from a real need, shared because others might need it too. Free to use, free to inspect, free forever.

---

## Project health

<p>
  <a href="https://github.com/rgdevment/LinkUnbound/actions/workflows/ci.yml">
    <img src="https://img.shields.io/github/actions/workflow/status/rgdevment/LinkUnbound/ci.yml?style=flat-square&logo=github-actions&label=Build" alt="Build Status"/>
  </a>
  <a href="https://sonarcloud.io/summary/overall?id=rgdevment_LinkUnbound">
    <img src="https://img.shields.io/sonar/quality_gate/rgdevment_LinkUnbound?server=https%3A%2F%2Fsonarcloud.io&style=flat-square&logo=sonarcloud&label=Quality%20Gate" alt="Quality Gate"/>
  </a>
  <a href="https://github.com/rgdevment/LinkUnbound/actions/workflows/mutants-sweep.yml">
    <img src="https://img.shields.io/endpoint?style=flat-square&url=https%3A%2F%2Fraw.githubusercontent.com%2Frgdevment%2FLinkUnbound%2Fscore%2Fmutants.json" alt="Mutation score"/>
  </a>
  <a href="https://dashboard.stryker-mutator.io/reports/github.com/rgdevment/LinkUnbound/main">
    <img src="https://img.shields.io/endpoint?style=flat-square&label=window%20logic&url=https%3A%2F%2Fbadge-api.stryker-mutator.io%2Fgithub.com%2Frgdevment%2FLinkUnbound%2Fmain" alt="Window mutation score"/>
  </a>
  <a href="https://sonarcloud.io/summary/overall?id=rgdevment_LinkUnbound">
    <img src="https://img.shields.io/sonar/coverage/rgdevment_LinkUnbound?server=https%3A%2F%2Fsonarcloud.io&style=flat-square&logo=sonarcloud&label=Coverage" alt="Coverage"/>
  </a>
</p>
