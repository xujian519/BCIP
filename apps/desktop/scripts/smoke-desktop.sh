#!/usr/bin/env bash
# 桌面端冒烟检查（M4）：构建 + bcip 探测（可选）
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

echo "==> prepare-sidecar (Tauri externalBin)"
bash scripts/prepare-sidecar.sh

echo "==> npm run lint"
npm run lint

echo "==> npm run build"
npm run build

echo "==> bcip CLI"
if command -v bcip >/dev/null 2>&1; then
  bcip --version
else
  echo "warn: bcip 不在 PATH，Tauri 启动后需 sidecar 或仅文件模式"
fi

echo "==> cargo check (src-tauri)"
(cd src-tauri && cargo check)

echo "==> app-server RPC (stdio)"
python3 scripts/smoke-app-server-rpc.py

if [[ "${BCIP_SMOKE_SKIP_E2E:-0}" != "1" ]]; then
  echo "==> Playwright E2E"
  npm run test:e2e:install
  npm run test:e2e
else
  echo "skip: Playwright E2E (BCIP_SMOKE_SKIP_E2E=1)"
fi

echo "ok: desktop smoke passed"
