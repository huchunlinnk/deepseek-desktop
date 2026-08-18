# deepseek-desktop

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](./LICENSE)

A thin native desktop shell for [DeepSeek Harness](https://github.com/deepseek-ai/deepseek-harness) — and the living target of the [`deepseek-desk-rsi`](https://github.com/huchunlinnk/deepseek-desk-rsi) engine. **AI for AI**: this app is *maintained* by a DeepSeek Harness agent that tracks upstream and opens a pull request every day.

## Why — the biggest advantages

1. **Native and light.** A ~9 MB Tauri shell that reuses the official host instead of reimplementing it — every harness capability is inherited for free.
2. **Self-maintained.** Every day a DeepSeek Harness agent (`deepseek-desk-rsi`) syncs it with upstream, enforces 1:1 feature parity, and opens a PR. You review and merge.
3. **Just works from Finder.** The host is vendored self-contained (absolute node path + relocatable launcher), so even a GUI launch with a minimal PATH boots it.
4. **Per-chip coverage.** macOS arm64/x64 and Windows x64/arm64 via CI.

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
# one-time: install deps, generate icons, and vendor the harness host
npm install
npm run tauri icon ./icon.png   # supply a 1024x1024 PNG
bash scripts/bundle-host.sh      # vendors @deepseek-ai/dsh into vendor/host/

# dev (run `dsh web` yourself, window shows the live UI)
npm run tauri dev

# production binary (bundles the vendored host, so no global `dsh` is needed)
npm run tauri build
```

The host command is overridable with `DSH_DESKTOP_HOST_CMD` (default `dsh web`). Resolution order: `DSH_DESKTOP_HOST_CMD` → bundled `vendor/host/dsh-launcher` → system `dsh`. The vendored launcher resolves the dsh bin relative to itself and bakes an absolute node path, so it works even from a Finder launch (whose PATH is minimal). Bundling the Node runtime itself (one per arch) is the remaining step for a fully self-contained binary; the launcher currently relies on a system `node`.

## How it stays current

`daily-upstream-sync.yml` runs a headless DSH agent every day that perceives upstream changes, integrates them, verifies (build + test), checks **1:1 feature parity** via `rsi_parity` against `parity.json`, repairs on failure, and opens a PR via `rsi_propose`. A human merges. See [PARITY.md](./PARITY.md) for the original-vs-desktop comparison, and the RSI engine repo for the loop semantics.

## License

MIT
