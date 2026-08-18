# 安全策略

## 报告漏洞

**不要**为安全漏洞开公开 issue。请通过 GitHub 的「报告漏洞」流程（Security → Advisories → New draft advisory）或 README 中指定的私密渠道私下报告。

请附上受影响版本、最小复现步骤和影响。

## 安全模型

`deepseek-desktop` 启动官方 DeepSeek Harness 宿主并在 webview 里展示。有两条信任边界：

1. **宿主是一个拥有 shell 权限的本地智能体。** 宿主能做什么，应用就能做什么。桌面壳不会在 harness 自身的沙箱和审批层之上再加隔离。
2. **密钥不进仓库。** API key 和 token 属于环境变量、GitHub Actions secrets 或 harness 凭据库——绝不在 `tauri.conf.json`、`parity.json` 或任何已提交文件里。

## 构建与运行时说明

- `scripts/bundle-host.sh` 在构建时从 npm vendor `@deepseek-ai/dsh`。钉住你 vendor 的版本，并在分发 release 前审计该包。
- `scripts/smoke-web.sh` 启动宿主并探测 `http://127.0.0.1:3080`；默认只绑回环地址。除非你理解暴露面，否则不要把 bind host 改成非回环地址。
- `daily-sync.yml` 跑一个能改仓库的智能体；给它最小权限（`contents:write` + `pull-requests:write`，除了 API key 和 `GITHUB_TOKEN` 不给别的 secret），并在 `main` 上保持分支保护。
