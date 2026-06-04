#!/usr/bin/env bash
# 本地打包：无 updater 私钥时跳过签名产物，避免 TAURI_SIGNING_PRIVATE_KEY 报错。
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "${ROOT}"

TAURI_ARGS=()
if [[ -n "${TAURI_SIGNING_PRIVATE_KEY:-}" ]] || [[ -f "${HOME}/.bcip/updater-key" ]]; then
  if [[ -z "${TAURI_SIGNING_PRIVATE_KEY:-}" ]]; then
    export TAURI_SIGNING_PRIVATE_KEY
    TAURI_SIGNING_PRIVATE_KEY="$(<"${HOME}/.bcip/updater-key")"
    export TAURI_SIGNING_PRIVATE_KEY
  fi
  echo "==> 已配置 updater 私钥，将生成签名更新包"
else
  TAURI_ARGS+=(--config '{"bundle":{"createUpdaterArtifacts":false}}')
  echo "==> 未找到 updater 私钥，跳过 createUpdaterArtifacts（本地打包）"
  echo "    发布版请设置 TAURI_SIGNING_PRIVATE_KEY 或运行: npm run updater:keygen"
fi

exec env -u RUSTC_WRAPPER CARGO_BUILD_RUSTC_WRAPPER= npx tauri build "${TAURI_ARGS[@]}" "$@"
