# deepseek-desktop

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](./LICENSE)

A thin native desktop shell for [DeepSeek Harness](https://github.com/deepseek-ai/deepseek-harness) — and the living target of the [`deepseek-desk-rsi`](https://github.com/huchunlinnk/deepseek-desk-rsi) engine. **AI for AI**: this app is *maintained* by a DeepSeek Harness agent that tracks upstream and opens a pull request every day.

## What it is

- Boots the official `dsh web` host (never reimplements the harness or the UI).
- Shows it in a native window with a system tray, native notifications, a global toggle shortcut, single-instance focus, and autostart.
- Plug-and-play update policy (`Pin` / `AutoBump` / `AutoBumpWithGate`) so the RSI engine can gate updates on its own verify pass.

## First principles / Occam's razor

- **One host, one viewer.** The shell spawns `dsh web` and points a webview at `http://127.0.0.1:3080`. Nothing else.
- **No frontend build.** The webview loads a static splash (`index.html`) and the Rust side redirects it to the host once ready. Zero JS tooling beyond the Tauri CLI.
- **Minimal glue.** Tray, shortcut, single-instance, autostart, host boot. That is the entire product.

## Platform matrix

| OS | Arch | Status |
|---|---|---|
| macOS | arm64 (Apple Silicon) | native |
| macOS | x64 (Intel) | native (macos-13 runner) |
| Windows | x64 | native |
| Windows | arm64 | experimental cross-compile — no native smoke |

## Build

```sh
# one-time: install deps and generate icons
npm install
npm run tauri icon ./icon.png   # supply a 1024x1024 PNG

# dev (run `dsh web` yourself, window shows the live UI)
npm run tauri dev

# production binary
npm run tauri build
```

The host command is overridable with `DSH_DESKTOP_HOST_CMD` (default `dsh web`). For a zero-environment install, vendor the host first and the shell will prefer it automatically:

```sh
bash scripts/bundle-host.sh   # vendors @deepseek-ai/dsh into vendor/host/
```

Resolution order: `DSH_DESKTOP_HOST_CMD` → bundled `vendor/host/dsh-launcher` → system `dsh`. Bundling the Node runtime itself (one per arch) is the remaining step for a fully self-contained binary; the launcher currently relies on a system `node`.

## How it stays current

`daily-upstream-sync.yml` runs a headless DSH agent every day that perceives upstream changes, integrates them, verifies (build + test), checks **1:1 feature parity** via `rsi_parity` against `parity.json`, repairs on failure, and opens a PR via `rsi_propose`. A human merges. See [PARITY.md](./PARITY.md) for the original-vs-desktop comparison, and the RSI engine repo for the loop semantics.

## License

MIT
