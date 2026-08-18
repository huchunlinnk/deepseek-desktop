# Changelog

All notable changes to this project are documented here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Thin Tauri 2 shell: host boot, system tray, native notifications, global toggle hotkey, single-instance, autostart.
- Pluggable update policy (`Pin` / `AutoBump` / `AutoBumpWithGate`) — Strategy pattern.
- Self-contained host vendoring (`scripts/bundle-host.sh`) with a relocatable launcher, so the app boots from a Finder launch with a minimal `PATH`.
- `scripts/smoke-web.sh` (serve smoke), `parity.json` + `PARITY.md` (the 1:1 feature-parity contract).
- Generated icon set (`src-tauri/icons/`) and an isolated frontend splash (`frontend/`).
- CI (`cargo check` / `cargo test`), the 4-arch release matrix, and the daily RSI sync workflow.
- English and Chinese READMEs.

### Fixed

- Global-shortcut registration no longer aborts startup on a plugin error type mismatch; it logs and continues.
- Host command resolution keeps paths with spaces intact (the `.app` bundle name was previously split on whitespace).
