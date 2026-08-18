# Releasing

Do these once before the first public release.

## 1. Org

This repo lives under `huchunlinnk`. If you fork it, replace `huchunlinnk` with your own user/org in:

- `README.md` (the `deepseek-desk-rsi` link)
- `.github/workflows/daily-sync.yml` (the `dsh plugin add` line)

## 2. Publish the repo

- Create the GitHub repo and push.
- Add the topics `dsh`, `dsh-plugin`, `deepseek-harness`.
- Ensure the icon set is committed (`src-tauri/icons/`, generated from `icon.png`).

## 3. Secrets (never committed)

GitHub → Settings → Secrets and variables → Actions, add:

| Name | Purpose |
|---|---|
| `DEEPSEEK_API_KEY` | runs the harness (chat + web search) |

`daily-sync.yml` uses the automatic `secrets.GITHUB_TOKEN` for pull requests — no manual token is needed.

## 4. Build and verify

```sh
npm install
npm run tauri icon ./icon.png   # regenerate icons if you change the source
bash scripts/bundle-host.sh     # vendor @deepseek-ai/dsh (required before bundling)
npm run tauri build
```

Push → `ci.yml` runs `cargo check --locked && cargo test --locked`. Tag `v0.1.0` → `release.yml` builds the 4-arch matrix (Windows arm64 is experimental).

## 5. First daily sync

Merge one PR, then confirm `daily-sync.yml` opens its first RSI pull request the next day. Keep branch protection on `main` so the loop can propose but never merge.
