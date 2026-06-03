#!/usr/bin/env bash
# 将 bcip 复制为 Tauri externalBin（命名：bcip-<host-triple>）
# 优先级：BCIP_SIDECAR_PATH > release > PATH bcip > debug（fallback）
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN_DIR="$ROOT/src-tauri/binaries"
mkdir -p "$BIN_DIR"

HOST="$(rustc -vV | sed -n 's/^host: //p')"
if [[ -z "${HOST}" ]]; then
  echo "prepare-sidecar: 无法读取 rustc host triple" >&2
  exit 1
fi

DEST="${BIN_DIR}/bcip-${HOST}"

resolve_bcip_source() {
  # 1. 环境变量显式指定
  if [[ -n "${BCIP_SIDECAR_PATH:-}" ]] && [[ -f "${BCIP_SIDECAR_PATH}" ]]; then
    echo "${BCIP_SIDECAR_PATH}"
    return 0
  fi

  local repo_root
  repo_root="$(cd "${ROOT}/../.." && pwd)"

  # 2. release build（优先）
  for candidate in \
    "${repo_root}/codex-rs/target/release/bcip" \
    "${repo_root}/codex-rs/target/release/codex"
  do
    if [[ -f "${candidate}" ]]; then
      echo "${candidate}"
      return 0
    fi
  done

  # 3. PATH bcip
  if command -v bcip >/dev/null 2>&1; then
    command -v bcip
    return 0
  fi

  # 4. debug build（fallback）
  for candidate in \
    "${repo_root}/codex-rs/target/debug/bcip" \
    "${repo_root}/codex-rs/target/debug/codex"
  do
    if [[ -f "${candidate}" ]]; then
      echo "prepare-sidecar: 警告 — 使用 debug 构建，建议先 cargo build --release" >&2
      echo "${candidate}"
      return 0
    fi
  done

  return 1
}

compress_with_upx() {
  local target="$1"
  if ! command -v upx >/dev/null 2>&1; then
    echo "prepare-sidecar: upx 未安装，跳过压缩" >&2
    return 1
  fi
  # UPX 5.x 在 macOS ARM 上需要 --force-macos（实验性支持，可能不稳定）
  local before after
  before=$(stat -f%z "${target}" 2>/dev/null || stat -c%s "${target}" 2>/dev/null || echo 0)
  if upx --best --force-macos "${target}" 2>/dev/null; then
    after=$(stat -f%z "${target}" 2>/dev/null || stat -c%s "${target}" 2>/dev/null || echo 0)
    local pct
    pct=$(awk "BEGIN { printf \"%.0f\", (1 - ${after}/${before}) * 100 }" 2>/dev/null || echo "?")
    echo "prepare-sidecar: UPX 压缩完成 ${before} → ${after} (${pct}%)"
    return 0
  else
    echo "prepare-sidecar: UPX 压缩失败（macOS 上可能不支持），保留原始二进制" >&2
    return 1
  fi
}

if src="$(resolve_bcip_source)"; then
  echo "prepare-sidecar: 复制 ${src} → ${DEST}"
  cp "${src}" "${DEST}"
  chmod +x "${DEST}"

  # UPX 压缩（可选，减小 50-70% 体积）
  if [[ "${BCIP_SKIP_UPX:-0}" != "1" ]]; then
    compress_with_upx "${DEST}" || true
  fi

  ls -lh "${DEST}"
else
  cat > "${DEST}" <<'EOF'
#!/bin/sh
# 开发占位：运行时委托 PATH 中的 bcip（请安装 bcip 或重新运行 prepare-sidecar）
exec bcip "$@"
EOF
  echo "prepare-sidecar: bcip 未找到，已生成 PATH 包装脚本 → ${DEST}"
  chmod +x "${DEST}"
fi
