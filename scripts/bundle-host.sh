#!/usr/bin/env bash
# Vendor the DeepSeek Harness host so the desktop can run with zero global `dsh`
# install and no runtime network access. Produces:
#   vendor/host/             node_modules with @deepseek-ai/dsh installed
#   vendor/host/dsh-launcher a relocatable wrapper that runs the vendored dsh bin
#
# The desktop's host.rs prefers this launcher (next to the executable) over the
# system `dsh`, unless DSH_DESKTOP_HOST_CMD overrides it.
set -euo pipefail

HOST_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/vendor/host"
# Resolve node to an absolute path up front: the launcher is spawned by a GUI
# process whose PATH is minimal, so it cannot rely on `env node` or bare `node`.
NODE_BIN="${NODE_BIN:-$(command -v node)}"
if [ -z "${NODE_BIN}" ]; then
  echo "[bundle-host] ERROR: node not found on PATH; set NODE_BIN=/abs/path/to/node" >&2
  exit 1
fi

mkdir -p "${HOST_DIR}"
cat > "${HOST_DIR}/package.json" <<'EOF'
{ "name": "dsh-host-vendor", "private": true, "type": "module" }
EOF

echo "[bundle-host] installing @deepseek-ai/dsh into ${HOST_DIR}"
(cd "${HOST_DIR}" && npm install --no-save --no-package-lock @deepseek-ai/dsh)

# Resolve the package's bin entry so the launcher stays correct across versions.
DSH_BIN_REL="$(DSH_HOST_DIR="${HOST_DIR}" "${NODE_BIN}" -e "
  const p = require(process.env.DSH_HOST_DIR + '/node_modules/@deepseek-ai/dsh/package.json')
  const bin = p.bin
  process.stdout.write(typeof bin === 'string' ? bin : Object.values(bin)[0])
")"

# Relocatable launcher: resolves the dsh bin relative to ITSELF, so it keeps
# working after the host is bundled into the app's Resources/ directory. Only
# the node binary path is baked (node is a system-level prerequisite).
cat > "${HOST_DIR}/dsh-launcher" <<EOF
#!/usr/bin/env bash
SELF_DIR="\$(cd "\$(dirname "\${BASH_SOURCE[0]}")" && pwd)"
exec "${NODE_BIN}" "\${SELF_DIR}/node_modules/@deepseek-ai/dsh/${DSH_BIN_REL}" "\$@"
EOF
chmod +x "${HOST_DIR}/dsh-launcher"

echo "[bundle-host] done: ${HOST_DIR}/dsh-launcher (node=${NODE_BIN})"
