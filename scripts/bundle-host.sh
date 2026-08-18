#!/usr/bin/env bash
# Vendor the DeepSeek Harness host so the desktop can run with zero global `dsh`
# install and no runtime network access. Produces:
#   vendor/host/             node_modules with @deepseek-ai/dsh installed
#   vendor/host/dsh-launcher a wrapper that runs the vendored dsh bin
#
# The desktop's host.rs prefers this launcher (next to the executable) over the
# system `dsh`, unless DSH_DESKTOP_HOST_CMD overrides it.
set -euo pipefail

HOST_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/vendor/host"
NODE_BIN="${NODE_BIN:-node}"

mkdir -p "${HOST_DIR}"
cat > "${HOST_DIR}/package.json" <<'EOF'
{ "name": "dsh-host-vendor", "private": true, "type": "module" }
EOF

echo "[bundle-host] installing @deepseek-ai/dsh into ${HOST_DIR}"
(cd "${HOST_DIR}" && npm install --no-save --no-package-lock @deepseek-ai/dsh)

# Resolve the package's bin entry so the launcher stays correct across versions.
DSH_BIN_REL="$(cd "${HOST_DIR}" && node -e "
  const p = require('./node_modules/@deepseek-ai/dsh/package.json')
  const bin = p.bin
  const rel = typeof bin === 'string' ? bin : Object.values(bin)[0]
  process.stdout.write(rel)
")"

cat > "${HOST_DIR}/dsh-launcher" <<EOF
#!/usr/bin/env bash
exec "${NODE_BIN}" "${HOST_DIR}/node_modules/@deepseek-ai/dsh/${DSH_BIN_REL}" "\$@"
EOF
chmod +x "${HOST_DIR}/dsh-launcher"

echo "[bundle-host] done: ${HOST_DIR}/dsh-launcher"
