# Releasing

Do these once before the first public release.

## 1. Set the org

Replace `YOUR_ORG` with your GitHub user or org in:

- `README.md` (the `deepseek-desk-rsi` link)
- `.github/workflows/daily-sync.yml` (the `dsh plugin add github:YOUR_ORG/...` line)

## 2. Publish the repo

- Create the GitHub repo and push.
- Add the topics `dsh`, `dsh-plugin`, `deepseek-harness`.
- Ensure the icon set is committed (`src-tauri/icons/`, generated from `icon.png`).

## 3. Secrets (never committed)

GitHub → Settings → Secrets and variables → Actions, add:

| Name | Purpose |
|---|---|
| `DEEPSEEK_API_KEY` | runs the harness (chat + web search) |
| `GH_TOKEN` | lets `daily-sync.yml` open pull requests (fine-grained PAT: `contents: write`, `pull-requests: write`) |

## 4. Build and verify

```sh
npm install
npm run tauri icon ./icon.png   # regenerate icons if you change the source
npm run tauri build
```

Push → `ci.yml` runs `cargo check --locked && cargo test --locked`. Tag `v0.1.0` → `release.yml` builds the 4-arch matrix (Windows arm64 is experimental).

## 5. First daily sync

Merge one PR, then confirm `daily-sync.yml` opens its first RSI pull request the next day. Keep branch protection on `main` so the loop can propose but never merge.
