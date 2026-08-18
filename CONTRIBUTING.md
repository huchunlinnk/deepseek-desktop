# Contributing

Thanks for helping build the desktop shell. This is the target of the `deepseek-desk-rsi` engine — a thin Tauri shell over the official DeepSeek Harness host.

## Development

```sh
npm install                          # Tauri CLI + tooling
npm run tauri icon ./icon.png        # regenerate platform icons (one-time)
bash scripts/bundle-host.sh          # vendor @deepseek-ai/dsh (one-time)
npm run tauri dev                    # dev (run `dsh web` yourself; window shows the live UI)
npm run tauri build                  # production binary (bundles the vendored host)
```

## Conventions

- **First principles:** the shell never reimplements the harness or the web UI; it boots the host and views it. Keep it thin.
- **Rust under `src-tauri/src/desktop/`** is the only host glue: `host.rs` (boot), `tray.rs`, `updater.rs` (Strategy), `mod.rs` (wiring).
- **Secrets never in source.** Use env vars / GitHub secrets / the harness credential store.
- **`cargo check --locked && cargo test --locked`** must pass (CI runs this; run it locally too).

## Submitting

1. Open an issue first for anything larger than a bug fix.
2. Branch, change, `cargo check`, `cargo test`.
3. Open a PR; keep it small and one-concern.
4. Tag the repo `dsh` and `dsh-plugin` so the community hub indexes it.
