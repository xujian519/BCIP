#!/bin/bash

# BCIP Agent 构建脚本（未签名测试版）
# 用于在 10 人以内进行内部测试
#
# 体积优化建议：
#   发布前用 codegen-units=1 构建以获得最小二进制：
#     CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo build --release -p codex-cli
#   UPX 在 macOS ARM 上不受支持，跳过压缩。

set -e

echo "🚀 BCIP Agent 构建开始..."
echo ""

# 检查必要工具
if ! command -v node &> /dev/null; then
    echo "❌ 错误：未找到 Node.js，请先安装"
    exit 1
fi

if ! command -v cargo &> /dev/null; then
    echo "❌ 错误：未找到 Rust/Cargo，请先安装"
    exit 1
fi

# 显示版本信息
echo "📦 Node.js 版本: $(node -v)"
echo "📦 npm 版本: $(npm -v)"
echo "📦 Rust 版本: $(rustc --version)"
echo ""

# 安装前端依赖
echo "📥 安装前端依赖..."
cd "$(dirname "$0")/../"
npm install

# 构建前端
echo "🔨 构建前端..."
npm run build

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

# 准备 bcip sidecar
echo "📦 准备 bcip sidecar..."
if ! bash "${SCRIPT_DIR}/prepare-sidecar.sh"; then
    echo "❌ 错误：prepare-sidecar.sh 执行失败，请先 cargo build --release -p codex-cli"
    exit 1
fi
echo "✅ bcip sidecar 就绪"

# 暂存知识库资产
echo "📦 暂存知识库数据文件..."
if ! bash "${SCRIPT_DIR}/prepare-assets.sh"; then
    echo "❌ 错误：prepare-assets.sh 执行失败，请检查知识库资产文件是否存在"
    exit 1
fi
echo "✅ 知识库资产暂存完成"

# 构建 Tauri 应用（含 bundle，以正确处理 resources 配置）
echo "🔨 构建 Tauri 应用并打包（未签名）..."
cd src-tauri
cargo tauri build

echo ""
echo "✅ 构建完成！"
echo ""
echo "📁 输出位置:"
echo "   - .app 包: src-tauri/target/release/bundle/macos/"
echo "   - .dmg 包: src-tauri/target/release/bundle/dmg/"
echo ""
echo "⚠️  注意：这是未签名版本，安装时需要在系统偏好设置中允许"
echo ""
echo "🎉 构建完成！可以将应用分发给测试人员了。"
