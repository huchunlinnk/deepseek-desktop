# Security Policy

## Reporting a vulnerability

Do **not** open a public issue for a security vulnerability. Report it privately via GitHub's **"Report a vulnerability"** flow (Security → Advisories → New draft advisory) or the private channel named in the README.

Include the affected version, a minimal reproduction, and the impact.

## Security model

`deepseek-desktop` boots the official DeepSeek Harness host and displays it in a webview. Two trust boundaries matter:

1. **The host is a local agent with shell access.** Whatever the host can do, the app can do. The desktop shell adds no isolation on top of the harness's own sandbox and approval layers.
2. **Secrets stay out of the repo.** API keys and tokens live in environment variables, GitHub Actions secrets, or the harness credential store — never in `tauri.conf.json`, `parity.json`, or committed files.

## Build and runtime notes

- `scripts/bundle-host.sh` vendors `@deepseek-ai/dsh` from npm at build time. Pin the version you vendor, and audit the package before distributing a release.
- `scripts/smoke-web.sh` boots the host and probes `http://127.0.0.1:3080`; it binds loopback by default. Do not change the bind host to a non-loopback address without understanding the exposure.
- The `daily-sync.yml` workflow runs an agent that can edit the repo; grant it the least privilege (contents:write + pull-requests:write, no secrets beyond the API key and `GH_TOKEN`), and keep branch protection on `main`.
