#!/usr/bin/env bash
# 全本机桌面打包：预拉依赖、禁用 sccache 远程/分布式，可选跳过 UPX。
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
REPO_ROOT="$(cd "${ROOT}/../.." && pwd)"
TAURI_DIR="${ROOT}/src-tauri"

echo "==> 本地打包模式（无 sccache wrapper，sparse 索引见 src-tauri/.cargo/config.toml）"

# 1. 图标（可跳过：SKIP_ICONS=1）
if [[ "${SKIP_ICONS:-0}" != "1" ]]; then
  python3 "${ROOT}/scripts/generate-macos-icons.py"
fi

# 2. 预编译 bcip sidecar（release，体积更小；已有则跳过）
if [[ "${SKIP_BCIP_BUILD:-0}" != "1" ]]; then
  if [[ ! -f "${REPO_ROOT}/codex-rs/target/release/bcip" ]]; then
    echo "==> 本地编译 bcip release（首次较慢，请耐心等待）"
    (
      cd "${REPO_ROOT}/codex-rs"
      env -u RUSTC_WRAPPER CARGO_BUILD_RUSTC_WRAPPER= \
        cargo build --release -p codex-cli --bin bcip
    )
  else
    echo "==> 已存在 codex-rs/target/release/bcip，跳过编译（设 FORCE_BCIP_BUILD=1 可强制重编）"
    if [[ "${FORCE_BCIP_BUILD:-0}" == "1" ]]; then
      (
        cd "${REPO_ROOT}/codex-rs"
        env -u RUSTC_WRAPPER CARGO_BUILD_RUSTC_WRAPPER= \
          cargo build --release -p codex-cli --bin bcip
      )
    fi
  fi
fi

# 3. 预下载 Tauri 依赖到本机（避免打包时才拉取/排队）
echo "==> cargo fetch（Tauri crate）"
(
  cd "${TAURI_DIR}"
  env -u RUSTC_WRAPPER CARGO_BUILD_RUSTC_WRAPPER= cargo fetch
)

# 4. sidecar + 前端 + Tauri bundle
export BCIP_SKIP_UPX="${BCIP_SKIP_UPX:-1}"
bash "${ROOT}/scripts/prepare-sidecar.sh"

(
  cd "${ROOT}"
  npm run build
  bash "${ROOT}/scripts/tauri-build.sh"
)

echo "==> 完成。产物见 ${TAURI_DIR}/target/release/bundle/"
