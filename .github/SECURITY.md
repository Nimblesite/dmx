<!-- agent-pmo:a72c926 -->
# Security Policy

GitHub surfaces this file on the repository's **Security** tab and on the
"Report a vulnerability" page. See GitHub's docs:

- Add a security policy: https://docs.github.com/en/code-security/how-tos/report-and-fix-vulnerabilities/configure-vulnerability-reporting/add-security-policy
- Configure private vulnerability reporting: https://docs.github.com/en/code-security/how-tos/report-and-fix-vulnerabilities/configure-vulnerability-reporting/configure-for-a-repository

## Reporting a Vulnerability

**Please do not report security vulnerabilities through public GitHub issues,
discussions, or pull requests.**

Report privately through GitHub's **private vulnerability reporting**: go to the
repository's **Security** tab → **Report a vulnerability** (or
https://github.com/Nimblesite/dmx/security/advisories/new). This
opens a private, structured advisory only the maintainers can see.

If you cannot use that channel, email **cftools@nimblesite.co**.

When reporting, please include:

- The type of issue (e.g. injection, path traversal, auth bypass, secret exposure).
- The affected version(s), file(s), and any relevant configuration.
- Steps to reproduce, ideally a minimal proof of concept.
- The impact: what an attacker can achieve.

`dmx` **rewrites Dart source files in place** and runs as a long-lived watcher
over a developer's working tree, so path traversal, symlink escape, and any
route to emitting attacker-controlled Dart into a file outside the target
directory are treated as high-severity.

## What to Expect

- **Acknowledgement** within **3 business days**.
- An assessment and a remediation plan (or a reasoned decline) within **10 business days**.
- Coordinated disclosure: we will agree a disclosure timeline with you and credit
  you in the advisory unless you prefer to remain anonymous.

## Supported Versions

`dmx` is pre-1.0. Security fixes land on the latest released version only; there
are no maintained older lines.

| Version | Supported |
| ------- | --------- |
| 0.1.x (latest) | ✅ |
| anything older | ❌ |
