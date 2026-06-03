#!/usr/bin/env bash
# 校验 apps/desktop/src/generated/app-server 与当前 bcip/codex 的 generate-ts 输出一致
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
DESKTOP="$(cd "$(dirname "$0")/.." && pwd)"
COMMITTED="${DESKTOP}/src/generated/app-server"
TMP="${DESKTOP}/.tmp-generate-ts-check"
CODEX_RS="${ROOT}/codex-rs"

resolve_cli() {
  if command -v bcip >/dev/null 2>&1; then
    command -v bcip
    return
  fi
  if command -v codex >/dev/null 2>&1; then
    command -v codex
    return
  fi
  local built="${CODEX_RS}/target/release/codex"
  if [[ -x "${built}" ]]; then
    echo "${built}"
    return
  fi
  built="${CODEX_RS}/target/debug/codex"
  if [[ -x "${built}" ]]; then
    echo "${built}"
    return
  fi
  return 1
}

rm -rf "${TMP}"
mkdir -p "${TMP}"

CLI="$(resolve_cli)" || {
  echo "check-generated-app-server: 未找到 bcip/codex，正在构建 codex-cli …" >&2
  (cd "${CODEX_RS}" && cargo build -q -p codex-cli --bin codex)
  CLI="$(resolve_cli)" || {
    echo "check-generated-app-server: 构建后仍无法找到 codex 二进制" >&2
    exit 1
  }
}

echo "==> ${CLI} app-server generate-ts"
"${CLI}" app-server generate-ts --out "${TMP}"

if ! diff -ru "${COMMITTED}" "${TMP}"; then
  echo "" >&2
  echo "generated 类型与仓库不一致。请在本机运行：" >&2
  echo "  bcip app-server generate-ts --out apps/desktop/src/generated/app-server" >&2
  echo "并提交 apps/desktop/src/generated/app-server 的变更。" >&2
  exit 1
fi

echo "ok: generated/app-server 与 generate-ts 输出一致"
rm -rf "${TMP}"
