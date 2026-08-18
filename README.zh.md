# deepseek-desktop

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](./LICENSE)

[DeepSeek Harness](https://github.com/deepseek-ai/deepseek-harness) 的原生桌面壳 —— 轻（约 9 MB）、薄，而且**能自我维护**。一个 DeepSeek Harness 智能体（[`deepseek-desk-rsi`](https://github.com/huchunlinnk/deepseek-desk-rsi)）每天追踪上游、强制 1:1 特性对等、并自动开 pull request。

**AI for AI：DSH 维护 DSH。**

## 为什么做这个

DeepSeek Harness 目前是 `0.1.x-rc`，频繁出破坏性变更。一个锁死某个版本的桌面壳几天就会腐坏。唯一可持续的答案是让应用**自己维护自己**——而 DSH 的核心主张就是"一切皆插件、agent 能自我进化"，所以维护者就是一个 DSH agent。

1. **原生、轻量。** 一个约 9 MB 的 Tauri 壳，复用官方宿主而不是重写它——宿主的一切能力（bash、文件系统、子代理、工作流、MCP、技能、计划模式……）全部免费继承。
2. **自我维护。** 每天 `daily-sync.yml` 会跑一个无头 DSH agent：感知上游变化 → 集成 → 验证（构建+测试+服务冒烟）→ 1:1 对等检查 → 失败修复 → 开 PR。你只负责 review 和 merge。
3. **从 Finder 双击就能跑。** 宿主被打成自包含（可重定位 launcher + 绝对 node 路径），即使 GUI 启动时的极简 `PATH` 也能拉起。
4. **分芯片全覆盖。** macOS arm64/x64 + Windows x64/arm64，由 CI 构建。

## 它是什么

- 启动官方 `dsh web` 宿主（绝不重写 harness 或 Web UI）。
- 用原生窗口展示它，带系统托盘、原生通知、全局切换快捷键（`Cmd/Ctrl+Shift+D`）、单实例聚焦、开机自启。
- 可插拔更新策略（`Pin` / `AutoBump` / `AutoBumpWithGate`），让 RSI 引擎能用自己的 verify 结果来把关更新。

## 架构

这个壳刻意做得很薄：只负责拉宿主 + 展示。

```
Tauri 壳（Rust）
 ├─ host.rs      拉 `dsh web`，轮询 :3080 直到就绪
 ├─ tray.rs      系统托盘（显示 / 退出）
 ├─ updater.rs   更新策略 —— Strategy 模式
 └─ mod.rs       装配（全局快捷键、单实例、自启）
        │  拉起
        ▼
 vendor/host/dsh-launcher   （vendor 的 @deepseek-ai/dsh，自包含）
        │
 webview ──► http://127.0.0.1:3080   （官方 DSH Web UI）
```

宿主解析顺序：`DSH_DESKTOP_HOST_CMD`（显式）→ 内置 `vendor/host/dsh-launcher` → 系统 `dsh`。

## 平台矩阵

| OS | 架构 | 状态 |
|---|---|---|
| macOS | arm64（Apple Silicon） | 原生 |
| macOS | x64（Intel） | 原生（macos-13 runner） |
| Windows | x64 | 原生 |
| Windows | arm64 | 实验性交叉编译 —— 无原生冒烟 |

## 快速开始

```sh
git clone https://github.com/huchunlinnk/deepseek-desktop.git
cd deepseek-desktop

npm install                      # Tauri CLI
npm run tauri icon ./icon.png    # 一次性；图标已提交
bash scripts/bundle-host.sh      # 把 @deepseek-ai/dsh vendor 进 vendor/host/
npm run tauri build              # 产出 .app（.dmg 在 CI 里产出）
```

构建产物在 `src-tauri/target/release/bundle/macos/DeepSeek Harness Desktop.app`。

## 它如何保持最新（AI-for-AI）

[`daily-sync.yml`](.github/workflows/daily-sync.yml) 每天跑一个无头 DSH agent：

```
perceive → integrate → verify → parity → repair（有界）→ propose
```

agent 对比上游 diff、改集成胶水、构建+测试+服务冒烟、校验 128 插件对等契约、失败则回滚重试（最多 3 轮）、最后带证据开 PR。由人 merge——这个环自己不会合代码。

## 特性对等

`parity.json` 列出了官方 `dsh-base` + `dsh-web-app` 两个 bundle 里的全部插件（128 个）。同步后只要少了任何一个名字，`rsi_parity` 就会失败。原版 vs 桌面版的对比见 [`PARITY.md`](./PARITY.md)。

## 仓库结构

```
frontend/         静态 splash（index.html）
scripts/          bundle-host.sh、smoke-web.sh
src-tauri/
  src/desktop/    host.rs、tray.rs、updater.rs、mod.rs
  icons/          生成的平台图标
.github/workflows/ ci.yml、daily-sync.yml、release.yml
parity.json       128 插件对等契约
```

## 安全

这个应用运行一个拥有 shell 权限的本地 agent；壳不会在 harness 自身的沙箱和审批层之上再加隔离。密钥绝不进仓库。见 [`SECURITY.md`](./SECURITY.md)。

## 许可证

[MIT](./LICENSE)
