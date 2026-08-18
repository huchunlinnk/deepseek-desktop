# Feature parity: DeepSeek Harness (original) vs deepseek-desktop

## The one principle

The desktop is a **thin shell over the official host**, never a reimplementation. It boots `dsh web`, which composes the `@deepseek-ai/dsh-base` + `@deepseek-ai/dsh-web-app` bundles — **128 plugins** that are the complete original feature surface.

Parity therefore means: **after every RSI sync, those 128 plugins are still present, and the web surface still serves.** The desktop reuses every one of them; it never re-adds them.

`parity.json` is the machine-readable form of this contract (auto-generated from the official bundle layers by `deepseek-desk-rsi/scripts/gen-parity.mjs`). The `rsi_parity` tool enforces it. `PARITY.md` (this file) is the human-readable comparison.

## The matrix

### 1. Runtime core — inherited (host)

| Capability | Original (`dsh web`) | Desktop | RSI verifies via |
|---|---|---|---|
| LLM adapters (deepseek, pi-ai) | `dsh-llm`, `dsh-llm-deepseek`, `dsh-llm-pi-ai` | identical (same host) | plugin name in `--dump-config` |
| Session persistence + replay | `dsh-session`, `dsh-session-persistence-jsonl` | identical | plugin name |
| Tool registry + schema | `dsh-tools`, `dsh-typert-*` | identical | plugin name |
| Agent + agent loop | `dsh-agent`, `dsh-agent-loop` | identical | plugin name |
| Jobs / background control | `dsh-jobs-local`, `dsh-tool-jobs` | identical | plugin name |
| Settings / credentials / identity | `dsh-settings-file`, `dsh-credentials-local` | identical | plugin name |
| Telemetry (OTLP) | `dsh-session-telemetry-otel` | identical | plugin name |

### 2. Capability seams — inherited (host)

| Capability | Original | Desktop | RSI verifies via |
|---|---|---|---|
| Shell (bash/pwsh) | `dsh-tool-bash`, `dsh-tool-pwsh`, `dsh-bash-sandbox` | identical | plugin name |
| Filesystem + observation | `dsh-tool-fs`, `dsh-tool-fs-search`, `dsh-fs-observation-policy` | identical | plugin name |
| Subprocess + sandbox | `dsh-subprocess-local`, `dsh-sandbox-local`, `dsh-fs-sandbox` | identical | plugin name |
| Web search / fetch | `dsh-web`, `dsh-web-search-deepseek`, `dsh-tool-web` | identical | plugin name |
| Subagents + control | `dsh-subagent`, `dsh-tool-subagent`, `dsh-tool-subagent-control` | identical | plugin name |
| Workflow (worker-thread) | `dsh-workflow-worker-thread`, `dsh-tool-workflow` | identical | plugin name |
| Skills + loader | `dsh-skill`, `dsh-tool-skill`, `dsh-skill-filesystem` | identical | plugin name |
| Goals + same-session rounds | `dsh-goal`, `dsh-goal-round-driver`, `dsh-tool-goal` | identical | plugin name |
| Plan mode | `dsh-plan-mode` | identical | plugin name |
| Compaction + pruning | `dsh-compaction-basic`, `dsh-compaction-tool-result-pruner` | identical | plugin name |
| Approval / permission presets | `dsh-user-approval`, `dsh-permission-presets` | identical | plugin name |

### 3. Model-facing tools — inherited (host, via presets)

`bash`, `pwsh`, `edit`/`read`/`read_image`/`write`, `glob`/`grep`, `web_search`/`web_fetch`, `subagent`/`subagent_fork`/`send_message`/`interrupt_agent`/`list_agents`, `workflow`, `skill`, `todo_write`, `exit_plan_mode`, `create_goal`/`get_goal`/`update_goal`, `ask_user_question`, `ralph`, `str_replace_editor`, `job_list`/`job_kill`/`job_output`, `run_code`.

Each is a `dsh-tool-*` plugin; parity checks their plugin names, so the tool surface cannot silently drop.

### 4. Web surface — inherited (host + browser roster)

| Capability | Original | Desktop | RSI verifies via |
|---|---|---|---|
| HTTP server + API gateway | `dsh-host-webserver`, `dsh-host-apiproxy`, `dsh-api-gateway` | identical | `scripts/smoke-web.sh` (HTTP 200) + plugin name |
| Client runtime + cordis | `dsh-client-runtime`, `dsh-cordis-client-runner`, `dsh-client-modules` | identical | plugin name |
| UI nodes (sidebar, settings, conversation, tool, plan, goal, workspace, jobs, presets, …) | `dsh-client-ui-*` (≈ 28 nodes) | identical | plugin name |
| Agent presets | `dsh-agent-presets` | identical | plugin name |
| Storage / feedback / workspace | `dsh-storage*`, `dsh-message-feedback`, `dsh-workspace` | identical | plugin name |
| Code runtime | `dsh-code-runtime-worker-thread` | identical | plugin name |

### 5. Desktop-only additive surface (not parity — additions on top)

| Feature | Desktop adds | RSI relationship |
|---|---|---|
| System tray (Show/Quit) | `src-tauri/src/desktop/tray.rs` | not a harness feature; unit-covered by the shell build |
| Native notifications | `tauri-plugin-notification` | additive |
| Global toggle hotkey | `desktop/mod.rs` | additive |
| Single-instance focus | `tauri-plugin-single-instance` | additive |
| Autostart | `tauri-plugin-autostart` | additive |
| Update policy (Pin/AutoBump/Gated) | `desktop/updater.rs` | additive; `Gated` defers to RSI verify |
| Host boot + readiness | `desktop/host.rs` | the shell's only contract with the host |

### 6. Out of scope (separate entry points, not part of the web profile)

The desktop wraps the **web** surface only. These are distinct DSH entry points and are not lost — they ship with the same host, just invoked differently:

- CLI one-shot (`dsh "task"`) and headless profile
- ACP server (automation protocol)
- JSON-RPC SDK server
- Python SDK

## Regenerate the contract

```sh
node deepseek-desk-rsi/scripts/gen-parity.mjs /path/to/deepseek-harness deepseek-desktop/parity.json
```

Re-run this whenever the RSI loop perceives an upstream plugin add/remove/rename, then re-run `rsi_parity`.
