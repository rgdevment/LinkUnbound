# Security Policy

## Security Matters

**LinkUnbound** intercepts every link you click on your system and decides which browser opens it. That's a sensitive position — I take it seriously.

This is a personal open source project, not a company product. Security here is built on transparency: the code is public, the policies are clear, and researchers are treated as partners.

---

## What We Do to Keep You Safe

### Privacy by Design

- **100% Local** — Your browser list, domain rules, and configuration never leave your machine. No cloud, no sync, no servers.
- **No Tracking** — No telemetry, no analytics, no usage data collection of any kind.
- **No Data Collection** — LinkUnbound stores only what it needs to work: browser paths, rules and preferences. Nothing else, and no log of the links it handles.

### Security Practices

- **Local JSON Storage** — Configuration is stored as plain JSON files in your user profile. No database, no network storage.
- **Minimal Permissions** — LinkUnbound reads the Windows registry to detect installed browsers and writes only its own browser registration and its own `linkunbound://` scheme, under the current user. It never writes the default-browser choice itself; Windows and macOS keep that for the person.
- **Open Source** — Every line of code is public under GPLv3. You can inspect, audit, and verify everything.
- **Dependency Updates** — Dependencies are regularly updated to patch known vulnerabilities.
- **Code Reviews** — All contributions go through review before merging.
- **No Link Log** — Links are routed in memory and never logged. The only addresses written to disk are the exact-address rules you ask it to remember, and the diagnostic report cuts those down to their host.

### Links From Other Applications

LinkUnbound answers for its own scheme, `linkunbound://open?url=<link>`, so an application can send it a link without LinkUnbound being the default browser. Any program on the machine, and any web page that gets the browser to follow it, can open that address, so it is treated as untrusted input:

- **One shape only** — the host must be `open` (in any case), with exactly one `url` parameter and no fragment. Any other spelling is dropped quietly: no window opens for it.
- **No hidden characters** — a link carrying tabs, line breaks, control characters or surrounding spaces is refused instead of cleaned up, and the one handed on is the link as the URL parser reads it, not the text that was sent.
- **Web links only** — the link inside must be `http` or `https` with a host. `file:`, UNC paths, `javascript:`, `data:`, settings pages, other schemes and a `linkunbound://` nested inside another are refused before anything is shown or launched.
- **Same guards as a click** — past that check the link goes through the same validation every link does: nothing that a browser would read as a command-line switch reaches it, and SafeLinks are unwrapped only when Microsoft serves them.
- **Nothing new is trusted** — a link that arrives this way decides nothing on its own. Your rules or the picker decide where it opens, as they would for a click. That cuts both ways: a page you allowed to open `linkunbound://` links can choose a link that one of your rules sends to another browser, profile or private window without asking. The browser asks before the first one, so only allow it for sites you trust.

The scheme is registered with the rest of the browser registration and removed with it: uninstalling removes it everywhere, and on Windows so does **Unregister** under Maintenance.

### Network Requests

LinkUnbound makes exactly **one type of network request** on its own: a read-only GET to the release feed the project publishes (`https://raw.githubusercontent.com/rgdevment/LinkUnbound/manifest/release-manifest.json`, and the channel file it names) to check for updates. No user data is sent, and the app works fully offline. Pressing **Update** then downloads the installer from this repository's releases page and verifies its signature against the key built into the app before running it; a Microsoft Store copy asks the Store instead.

---

## Supported Versions

| Version          | Supported                              |
| :--------------- | :------------------------------------: |
| 2.x              | Yes                                    |
| 1.4.x            | Security and crash fixes only          |
| Older than 1.4.x | No                                     |

The 2.x line is `main`. The 1.4.x line lives on the `v1-stable` branch and receives security and crash fixes only — no new features. Always use the [latest version](https://github.com/rgdevment/LinkUnbound/releases/latest).

---

## Reporting a Vulnerability

If you discover a security vulnerability in LinkUnbound, please report it responsibly.

### What Qualifies

**Please report:**

- Unauthorized access to stored configuration or rules
- Privilege escalation issues
- URL injection or manipulation that could redirect to unintended targets
- Anything other than an `http` or `https` link getting through `linkunbound://`
- Bypass of domain rule matching
- Command injection through browser arguments or URL handling
- Data leakage or unintended storage
- A path, query or credential from a rule surviving into the diagnostic report

**Not security issues:**

- Feature requests or enhancements
- General bugs without security implications
- Windows SmartScreen warnings (expected for self-signed installers)

### How to Report

**Do not open a public GitHub issue for security vulnerabilities.**

**Email:** [github@apirest.cl](mailto:github@apirest.cl)

Subject: `[SECURITY] Brief description of the issue`

**Include:**

- Description of the vulnerability
- Impact — what could an attacker do?
- Steps to reproduce
- LinkUnbound version, and your OS and version (Windows 10/11 or macOS)
- Proof of concept (optional)
- Suggested fix (optional)

### What Happens Next

1. **Acknowledgment (48 hours)** — I confirm receipt and ask for more info if needed.
2. **Investigation (1-7 days)** — I reproduce, assess severity, and develop a fix.
3. **Resolution** — Patch, test, coordinate release timeline with you.
4. **Disclosure** — Security advisory published after fix is released.

### Response Times

| Severity | Response | Fix Target |
| :------- | :------: | :--------: |
| Critical | 24 hours | 1-3 days   |
| High     | 48 hours | 3-7 days   |
| Medium   | 3 days   | 1-2 weeks  |
| Low      | 1 week   | Next release |

I'm one developer. If you don't hear back within the expected timeframe, please follow up.

---

## Responsible Disclosure

I believe in coordinated disclosure:

- Please give reasonable time to fix the issue before public disclosure.
- I aim to release fixes within 7 days for critical issues.
- I will credit you in the release notes (unless you prefer anonymity).

**I will:**

- Treat you with respect — you're helping protect users.
- Respond promptly and keep you updated.
- Credit your work publicly (if you want).

**I will never:**

- Threaten legal action against good-faith researchers.
- Ignore or dismiss legitimate reports.
- Retaliate against reporters.

---

## Questions

- **Security questions:** [github@apirest.cl](mailto:github@apirest.cl)
- **General questions:** [GitHub Discussions](https://github.com/rgdevment/LinkUnbound/discussions)
- **Vulnerability reports:** Use the private channels above — never post security issues publicly.

If you're unsure whether something is a security issue, reach out anyway. Better to have a conversation than miss a real problem.
