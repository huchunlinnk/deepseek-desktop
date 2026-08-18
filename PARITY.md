# 特性对等：DeepSeek Harness（原版）vs deepseek-desktop

## 唯一原则

桌面端是**官方宿主之上的薄壳**，绝非重写。它启动 `dsh web`，后者组合了 `@deepseek-ai/dsh-base` + `@deepseek-ai/dsh-web-app` 两个 bundle —— **128 个插件**，就是完整的原始特性面。

因此"对等"意味着：**每次 RSI 同步之后，那 128 个插件都还在，而且 Web 表面还能正常服务。** 桌面端复用其中每一个；它从不重新添加它们。

`parity.json` 是这份契约的机器可读形式（由 `deepseek-desk-rsi/scripts/gen-parity.mjs` 从官方 bundle 层自动生成）。`rsi_parity` 工具负责执行它。本文件 `PARITY.md` 是人可读的对比。

## 对比矩阵

### 1. 运行时核心 —— 继承（宿主）

| 能力 | 原版（`dsh web`） | 桌面端 | RSI 如何验证 |
|---|---|---|---|
| LLM 适配器（deepseek、pi-ai） | `dsh-llm`、`dsh-llm-deepseek`、`dsh-llm-pi-ai` | 一致（同一宿主） | `--dump-config` 里的插件名 |
| 会话持久化 + 回放 | `dsh-session`、`dsh-session-persistence-jsonl` | 一致 | 插件名 |
| 工具注册表 + schema | `dsh-tools`、`dsh-typert-*` | 一致 | 插件名 |
| Agent + agent loop | `dsh-agent`、`dsh-agent-loop` | 一致 | 插件名 |
| 任务 / 后台控制 | `dsh-jobs-local`、`dsh-tool-jobs` | 一致 | 插件名 |
| 设置 / 凭据 / 身份 | `dsh-settings-file`、`dsh-credentials-local` | 一致 | 插件名 |
| 遥测（OTLP） | `dsh-session-telemetry-otel` | 一致 | 插件名 |

### 2. 能力缝 —— 继承（宿主）

| 能力 | 原版 | 桌面端 | RSI 如何验证 |
|---|---|---|---|
| Shell（bash/pwsh） | `dsh-tool-bash`、`dsh-tool-pwsh`、`dsh-bash-sandbox` | 一致 | 插件名 |
| 文件系统 + 观测 | `dsh-tool-fs`、`dsh-tool-fs-search`、`dsh-fs-observation-policy` | 一致 | 插件名 |
| 子进程 + 沙箱 | `dsh-subprocess-local`、`dsh-sandbox-local`、`dsh-fs-sandbox` | 一致 | 插件名 |
| 网页搜索 / 抓取 | `dsh-web`、`dsh-web-search-deepseek`、`dsh-tool-web` | 一致 | 插件名 |
| 子代理 + 控制 | `dsh-subagent`、`dsh-tool-subagent`、`dsh-tool-subagent-control` | 一致 | 插件名 |
| 工作流（worker-thread） | `dsh-workflow-worker-thread`、`dsh-tool-workflow` | 一致 | 插件名 |
| 技能 + 加载器 | `dsh-skill`、`dsh-tool-skill`、`dsh-skill-filesystem` | 一致 | 插件名 |
| 目标 + 同会话轮次 | `dsh-goal`、`dsh-goal-round-driver`、`dsh-tool-goal` | 一致 | 插件名 |
| 计划模式 | `dsh-plan-mode` | 一致 | 插件名 |
| 压缩 + 裁剪 | `dsh-compaction-basic`、`dsh-compaction-tool-result-pruner` | 一致 | 插件名 |
| 审批 / 权限预设 | `dsh-user-approval`、`dsh-permission-presets` | 一致 | 插件名 |

### 3. 模型可见工具 —— 继承（宿主，经预设）

`bash`、`pwsh`、`edit`/`read`/`read_image`/`write`、`glob`/`grep`、`web_search`/`web_fetch`、`subagent`/`subagent_fork`/`send_message`/`interrupt_agent`/`list_agents`、`workflow`、`skill`、`todo_write`、`exit_plan_mode`、`create_goal`/`get_goal`/`update_goal`、`ask_user_question`、`ralph`、`str_replace_editor`、`job_list`/`job_kill`/`job_output`、`run_code`。

每一个都是一个 `dsh-tool-*` 插件；对等检查比对它们的插件名，所以工具面不可能悄悄丢失。

### 4. Web 表面 —— 继承（宿主 + 浏览器插件清单）

| 能力 | 原版 | 桌面端 | RSI 如何验证 |
|---|---|---|---|
| HTTP 服务器 + API 网关 | `dsh-host-webserver`、`dsh-host-apiproxy`、`dsh-api-gateway` | 一致 | `scripts/smoke-web.sh`（HTTP 200）+ 插件名 |
| 客户端运行时 + cordis | `dsh-client-runtime`、`dsh-cordis-client-runner`、`dsh-client-modules` | 一致 | 插件名 |
| UI 节点（侧栏、设置、对话、工具、计划、目标、工作区、任务、预设……） | `dsh-client-ui-*`（约 28 个节点） | 一致 | 插件名 |
| Agent 预设 | `dsh-agent-presets` | 一致 | 插件名 |
| 存储 / 反馈 / 工作区 | `dsh-storage*`、`dsh-message-feedback`、`dsh-workspace` | 一致 | 插件名 |
| 代码运行时 | `dsh-code-runtime-worker-thread` | 一致 | 插件名 |

### 5. 桌面端独有增量（非对等 —— 在原版之上的增量）

| 特性 | 桌面端新增 | 与 RSI 的关系 |
|---|---|---|
| 系统托盘（显示/退出） | `src-tauri/src/desktop/tray.rs` | 非 harness 特性；由壳构建的单元测试覆盖 |
| 原生通知 | `tauri-plugin-notification` | 增量 |
| 全局切换快捷键 | `desktop/mod.rs` | 增量 |
| 单实例聚焦 | `tauri-plugin-single-instance` | 增量 |
| 开机自启 | `tauri-plugin-autostart` | 增量 |
| 更新策略（Pin/AutoBump/Gated） | `desktop/updater.rs` | 增量；`Gated` 交由 RSI verify 把关 |
| 宿主启动 + 就绪 | `desktop/host.rs` | 壳与宿主的唯一契约 |

### 6. 范围外（独立入口，不属于 web profile）

桌面端只包裹 **web** 表面。下面这些是独立的 DSH 入口，不会丢——它们随同一个宿主发布，只是调用方式不同：

- CLI 一次性（`dsh "task"`）和无头 profile
- ACP 服务器（自动化协议）
- JSON-RPC SDK 服务器
- Python SDK

## 重新生成契约

```sh
node deepseek-desk-rsi/scripts/gen-parity.mjs /path/to/deepseek-harness deepseek-desktop/parity.json
```

每当 RSI 环感知到上游插件增删改名时重跑一次，然后再跑 `rsi_parity`。
