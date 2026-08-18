# 发布

首次公开发布前做一次下面这些。

## 1. Org

本仓库位于 `huchunlinnk`。如果你 fork 了它，把 `huchunlinnk` 换成你自己的用户/组织，改：

- `README.md`（`deepseek-desk-rsi` 链接）
- `.github/workflows/daily-sync.yml`（`dsh plugin add` 那行）

## 2. 发布仓库

- 建 GitHub 仓库并推送。
- 加 topic `dsh`、`dsh-plugin`、`deepseek-harness`。
- 确认图标集已提交（`src-tauri/icons/`，由 `icon.png` 生成）。

## 3. Secrets（绝不提交）

GitHub → Settings → Secrets and variables → Actions，加：

| 名称 | 用途 |
|---|---|
| `DEEPSEEK_API_KEY` | 运行 harness（对话 + 网页搜索） |

`daily-sync.yml` 用自动的 `secrets.GITHUB_TOKEN` 开 PR——不需要手动 token。

## 4. 构建与验证

```sh
npm install
npm run tauri icon ./icon.png   # 改了源图标就重新生成
bash scripts/bundle-host.sh     # vendor @deepseek-ai/dsh（打包前必须）
npm run tauri build
```

推送 → `ci.yml` 跑 `cargo check --locked && cargo test --locked`。打 `v0.1.0` 标签 → `release.yml` 构建 4 架构矩阵（Windows arm64 为实验性）。

## 5. 首次每日同步

合并一个 PR，然后确认第二天 `daily-sync.yml` 开出它的第一个 RSI pull request。在 `main` 上保持分支保护，让环只能提议、不能合并。
