#!/usr/bin/env bash
# 初始化 ~/.bcip，与 Codex 桌面 ~/.codex 隔离
set -euo pipefail

BCIP_HOME="${BCIP_HOME:-${HOME}/.bcip}"
CONFIG="${BCIP_HOME}/config.toml"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TEMPLATE="${ROOT}/scripts/config/bcip-default-config.toml"

echo "==> BCIP 配置目录: ${BCIP_HOME}"
mkdir -p "${BCIP_HOME}/skills"

# 复制 constitutional 规则文件到 BCIP_HOME
ASSETS_SRC="${ROOT}/codex-rs/codex-patent-assets"
ASSETS_DST="${BCIP_HOME}/codex-patent-assets"
if [[ -d "${ASSETS_SRC}/constitutional" && ! -d "${ASSETS_DST}/constitutional" ]]; then
  mkdir -p "${ASSETS_DST}"
  cp -r "${ASSETS_SRC}/constitutional" "${ASSETS_DST}/constitutional"
  echo "    已复制 constitutional 规则文件"
fi

if [[ -f "${CONFIG}" ]]; then
  echo "    已存在 config.toml，跳过写入"
else
  if [[ -f "${TEMPLATE}" ]]; then
    cp "${TEMPLATE}" "${CONFIG}"
    echo "    已从模板创建 config.toml"
  else
    cat > "${CONFIG}" <<'EOF'
# BCIP Agent 专用配置（~/.bcip）
model_provider = "OpenAI"
model = "glm-5.1"
model_reasoning_effort = "medium"
disable_response_storage = true
model_context_window = 1000000
model_auto_compact_token_limit = 900000

[model_providers.OpenAI]
requires_openai_auth = true
wire_api = "responses"
base_url = "http://127.0.0.1:8788/v1"
name = "OpenAI"

[features]
js_repl = false
EOF
    echo "    已创建默认 config.toml"
  fi
fi

echo ""
echo "完成。BCIP 使用独立目录，不会读取 ~/.codex。"
echo "  配置: ${CONFIG}"
echo "  请在 shell 中设置:"
echo "    export BCIP_HOME=\"${BCIP_HOME}\""
echo "    export CODEX_HOME=\"${BCIP_HOME}\""
