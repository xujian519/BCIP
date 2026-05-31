#!/usr/bin/env bash
# 本地开发：编译 bcip、配置 PATH、刷新桌面 sidecar
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CODEX_RS="${ROOT}/codex-rs"
BIN_DIR="${CODEX_RS}/target/debug"
BCIP_BIN="${BIN_DIR}/bcip"
CODEX_BIN="${BIN_DIR}/codex"
ZSHRC="${HOME}/.zshrc"
PATH_LINE="export PATH=\"${BIN_DIR}:\$PATH\""

BCIP_HOME="${BCIP_HOME:-${HOME}/.bcip}"
ENV_BLOCK="# BCIP Agent（与 Codex 桌面 ~/.codex 隔离）
export BCIP_HOME=\"${BCIP_HOME}\"
export CODEX_HOME=\"${BCIP_HOME}\""

echo "==> BCIP 开发 CLI 配置"
echo "    仓库: ${ROOT}"
echo "    配置目录: ${BCIP_HOME}"

bash "${ROOT}/scripts/bootstrap-bcip-home.sh"

if [[ ! -f "${CODEX_BIN}" ]]; then
  echo "==> 未找到 codex，开始编译（需已设置 LK_CUSTOM_WEBRTC 或已下载 WebRTC）"
  if [[ -d "${HOME}/webrtc-prebuilt/mac-arm64-release" ]]; then
    export LK_CUSTOM_WEBRTC="${HOME}/webrtc-prebuilt/mac-arm64-release"
    echo "    使用 LK_CUSTOM_WEBRTC=${LK_CUSTOM_WEBRTC}"
  fi
  (cd "${CODEX_RS}" && cargo build --bin codex)
fi

ln -sf codex "${BCIP_BIN}"
echo "==> bcip → codex 软链: ${BCIP_BIN}"
"${BCIP_BIN}" --version

if ! grep -Fq "${BIN_DIR}" "${ZSHRC}" 2>/dev/null; then
  {
    echo ""
    echo "# BCIP Agent 本地开发 CLI"
    echo "${PATH_LINE}"
  } >> "${ZSHRC}"
  echo "==> 已写入 PATH 到 ~/.zshrc"
else
  echo "==> ~/.zshrc 已包含 codex-rs/target/debug，跳过 PATH"
fi

if ! grep -Fq 'BCIP Agent（与 Codex 桌面' "${ZSHRC}" 2>/dev/null; then
  {
    echo ""
    echo "${ENV_BLOCK}"
  } >> "${ZSHRC}"
  echo "==> 已写入 BCIP_HOME / CODEX_HOME 到 ~/.zshrc"
else
  echo "==> ~/.zshrc 已包含 BCIP_HOME，跳过"
fi

echo "==> 请执行: source ~/.zshrc"

echo "==> 刷新桌面 sidecar"
bash "${ROOT}/apps/desktop/scripts/prepare-sidecar.sh"

file "${ROOT}/apps/desktop/src-tauri/binaries/bcip-$(rustc -vV | sed -n 's/^host: //p')" \
  | grep -q 'Mach-O' && echo "==> sidecar OK（Mach-O）" || {
  echo "ERROR: sidecar 仍是脚本，请检查 codex 是否编译成功" >&2
  exit 1
}

echo ""
echo "完成。新开终端后可直接运行: bcip"
echo "启动桌面: cd ${ROOT} && npm run tauri:dev"
