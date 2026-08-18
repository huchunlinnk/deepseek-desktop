# Changelog

All notable changes to this project are documented here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Thin Tauri 2 shell: host boot (`dsh web`), tray, native notifications, global toggle hotkey, single-instance, autostart, pluggable update policy (Strategy).
- Host sidecar vendoring (`scripts/bundle-host.sh`) with env → bundled → system resolution.
- `scripts/smoke-web.sh` serving smoke; `parity.json` + `PARITY.md` parity contract.
- CI (cargo check/test), 4-arch release matrix, and the daily RSI sync workflow.
