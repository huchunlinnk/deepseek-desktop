# 贡献指南

感谢你为桌面壳出力。它是 `deepseek-desk-rsi` 引擎的目标——官方 DeepSeek Harness 宿主之上的一个薄 Tauri 壳。

## 开发

```sh
npm install                          # Tauri CLI + 工具链
npm run tauri icon ./icon.png        # 重新生成平台图标（一次性）
bash scripts/bundle-host.sh          # vendor @deepseek-ai/dsh（一次性）
npm run tauri dev                    # 开发（自己跑 `dsh web`；窗口显示实时 UI）
npm run tauri build                  # 生产二进制（打包内置宿主）
```

## 约定

- **第一性原则：** 这个壳绝不重写 harness 或 Web UI；它只拉宿主并展示。保持薄。
- **`src-tauri/src/desktop/` 下的 Rust** 是唯一的宿主胶水：`host.rs`（启动）、`tray.rs`、`updater.rs`（Strategy）、`mod.rs`（装配）。
- **密钥绝不进源码。** 用环境变量 / GitHub secrets / harness 凭据库。
- **`cargo check --locked && cargo test --locked`** 必须通过（CI 会跑；本地也跑）。

## 提交

1. 比 bug 修复更大的改动，先开 issue。
2. 开分支、改、`cargo check`、`cargo test`。
3. 开 PR；保持小而单一关注点。
4. 给仓库打 `dsh` 和 `dsh-plugin` topic，便于社区索引。
