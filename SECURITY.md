# Security Policy

## Security Matters

**LinkUnbound** intercepts every link you click on your system and decides which browser opens it. That's a sensitive position — I take it seriously.

This is a personal open source project, not a company product. Security here is built on transparency: the code is public, the policies are clear, and researchers are treated as partners.

---

## What We Do to Keep You Safe

### Privacy by Design

- **100% Local** — Your browser list, domain rules, and configuration never leave your machine. No cloud, no sync, no servers.
- **No Tracking** — No telemetry, no analytics, no usage data collection of any kind.
- **No Data Collection** — LinkUnbound stores only what it needs to work: browser paths, domain rules, and a navigation log. Nothing else.

### Security Practices

- **Local JSON Storage** — Configuration is stored as plain JSON files in your user profile. No database, no network storage.
- **Minimal Permissions** — LinkUnbound only reads the Windows registry to detect installed browsers. It does not modify system settings beyond its own browser registration.
- **Open Source** — Every line of code is public under GPLv3. You can inspect, audit, and verify everything.
- **Dependency Updates** — Dependencies are regularly updated to patch known vulnerabilities.
- **Code Reviews** — All contributions go through review before merging.
- **URL Redaction in Logs** — All URLs are automatically redacted at write time before reaching the log file. The `navigate.log` never contains actual URLs, only privacy-safe placeholders.

### Network Requests

LinkUnbound makes exactly **one type of network request**: a read-only GET to the GitHub Releases API (`https://api.github.com/repos/rgdevment/LinkUnbound/releases/latest`) to check for updates. No user data is sent. The app works fully offline.

---

## Supported Versions

| Version        | Supported |
| :------------- | :-------: |
| Latest Release | Yes       |
| Older Releases | No        |

Always use the [latest version](https://github.com/rgdevment/LinkUnbound/releases/latest).

---

## Reporting a Vulnerability

If you discover a security vulnerability in LinkUnbound, please report it responsibly.

### What Qualifies

**Please report:**

- Unauthorized access to stored configuration or rules
- Privilege escalation issues
- URL injection or manipulation that could redirect to unintended targets
- Bypass of domain rule matching
- Command injection through browser arguments or URL handling
- Data leakage or unintended storage
- Bypass of URL redaction in log service

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
