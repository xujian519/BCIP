#!/usr/bin/env bash
# 将知识库数据文件暂存到 Tauri 资源目录，供打包时捆绑进 .app 包
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
ASSETS_SRC="${REPO_ROOT}/codex-rs/codex-patent-assets"
ASSETS_DST="${REPO_ROOT}/apps/desktop/src-tauri/target/codex-patent-assets"

if [[ ! -d "${ASSETS_SRC}" ]]; then
    echo "Warning: codex-patent-assets not found at ${ASSETS_SRC}, skipping"
    exit 0
fi

echo "==> Staging knowledge assets for bundling..."
mkdir -p "${ASSETS_DST}"

# 核心数据文件
for f in patent_kg.db laws.db card-index.json; do
    if [[ -f "${ASSETS_SRC}/${f}" ]]; then
        cp -a "${ASSETS_SRC}/${f}" "${ASSETS_DST}/${f}"
        size=$(du -h "${ASSETS_DST}/${f}" | cut -f1)
        echo "    ${f} (${size})"
    fi
done

# 目录资产
for d in cards constitutional rules; do
    if [[ -d "${ASSETS_SRC}/${d}" ]]; then
        rm -rf "${ASSETS_DST}/${d}"
        cp -a "${ASSETS_SRC}/${d}" "${ASSETS_DST}/${d}"
        echo "    ${d}/"
    fi
done

echo "==> Assets staged at ${ASSETS_DST}"
