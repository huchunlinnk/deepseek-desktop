# deepseek-desktop

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](./LICENSE)

A native desktop shell for [DeepSeek Harness](https://github.com/deepseek-ai/deepseek-harness) — thin, light (~9 MB), and **self-maintaining**. A DeepSeek Harness agent ([`deepseek-desk-rsi`](https://github.com/huchunlinnk/deepseek-desk-rsi)) tracks upstream every day, enforces 1:1 feature parity, and opens a pull request.

**AI for AI: DSH maintains DSH.**

## Why this exists

DeepSeek Harness is in `0.1.x-rc` and ships breaking changes often. A desktop shell that pins one version rots within days. The only sustainable answer is to make the app *maintain itself* — and since DSH's whole thesis is "everything is a plugin, the agent can evolve itself", the maintainer is a DSH agent.

1. **Native and light.** A ~9 MB Tauri shell that reuses the official host instead of reimplementing it — every harness capability (bash, filesystem, subagents, workflows, MCP, skills, plan mode, …) is inherited for free.
2. **Self-maintained.** Every day the `daily-sync.yml` workflow runs a headless DSH agent that perceives upstream changes, integrates them, verifies (build + test + serve), checks 1:1 parity, repairs on failure, and opens a PR. You review and merge.
3. **Just works from Finder.** The host is vendored self-contained — a relocatable launcher with an absolute node path — so even a GUI launch with a minimal `PATH` boots it.
4. **Per-chip coverage.** macOS arm64/x64 and Windows x64/arm64, built by CI.

## What it is

- Boots the official `dsh web` host (never reimplements the harness or the web UI).
- Shows it in a native window with a system tray, native notifications, a global toggle shortcut (`Cmd/Ctrl+Shift+D`), single-instance focus, and autostart.
- Pluggable update policy (`Pin` / `AutoBump` / `AutoBumpWithGate`) so the RSI engine can gate updates on its own verify pass.

## Architecture

The shell is deliberately thin: it only boots the host and views it.

```
Tauri shell (Rust)
 ├─ host.rs      spawn `dsh web`, poll :3080 until ready
 ├─ tray.rs      system tray (Show / Quit)
 ├─ updater.rs   update policy — Strategy pattern
 └─ mod.rs       wiring (global shortcut, single-instance, autostart)
        │  spawns
        ▼
 vendor/host/dsh-launcher   (vendored @deepseek-ai/dsh, self-contained)
        │
 webview ──► http://127.0.0.1:3080   (the official DSH web UI)
```

Host resolution order: `DSH_DESKTOP_HOST_CMD` (explicit) → bundled `vendor/host/dsh-launcher` → system `dsh`.

## Platform matrix

| OS | Arch | Status |
|---|---|---|
| macOS | arm64 (Apple Silicon) | native |
| macOS | x64 (Intel) | native (macos-13 runner) |
| Windows | x64 | native |
| Windows | arm64 | experimental cross-compile — no native smoke |

## Quick start

```sh
git clone https://github.com/huchunlinnk/deepseek-desktop.git
cd deepseek-desktop

npm install                      # Tauri CLI
npm run tauri icon ./icon.png    # one-time; icons are already committed
bash scripts/bundle-host.sh      # vendor @deepseek-ai/dsh into vendor/host/
npm run tauri build              # produce the .app (and .dmg in CI)
```

The built app lands at `src-tauri/target/release/bundle/macos/DeepSeek Harness Desktop.app`.

## How it stays current (AI-for-AI)

[`daily-sync.yml`](.github/workflows/daily-sync.yml) runs a headless DSH agent every day:

```
perceive → integrate → verify → parity → repair (bounded) → propose
```

The agent diffs upstream, edits the integration glue, builds + tests + smokes the serve path, checks the 128-plugin parity contract, rolls back and retries on failure (max 3 rounds), and opens a PR with the evidence. A human merges — the loop never lands code on its own.

## Feature parity

`parity.json` lists every plugin in the official `dsh-base` + `dsh-web-app` bundles (128 plugins). `rsi_parity` fails if any name goes missing after a sync. See [`PARITY.md`](./PARITY.md) for the original-vs-desktop comparison.

## Repository layout

```
frontend/          static splash (index.html)
scripts/           bundle-host.sh, smoke-web.sh
src-tauri/
  src/desktop/     host.rs, tray.rs, updater.rs, mod.rs
  icons/           generated platform icons
.github/workflows/ ci.yml, daily-sync.yml, release.yml
parity.json        the 128-plugin parity contract
```

## Security

This app runs a local agent with shell access; the shell adds no isolation on top of the harness's own sandbox and approval layers. Secrets never live in the repo. See [`SECURITY.md`](./SECURITY.md).

## License

[MIT](./LICENSE)
