#!/usr/bin/env bash
# BCIP 本地开发：最小增量编译（不 clean、不编 webrtc-sys、不编全 workspace）
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
RS="${ROOT}/codex-rs"
cd "$RS"

if pgrep -x cargo >/dev/null 2>&1; then
  echo "提示: 检测到其他 cargo 进程，会抢 lock 导致极慢。请先结束其它 cargo build/check。" >&2
fi

# 只编 CLI 一次，并启用 patent-tools（勿先单独编无 feature 的 codex-core，否则会命中错误缓存）
echo "==> 编译 bcip（default 含 patent-tools，已关闭 WebRTC）…"
touch cli/src/main.rs
cargo build -p codex-cli --bin bcip

BIN="${RS}/target/debug/bcip"
if [[ ! -f "$BIN" ]]; then
  echo "错误: 未生成 ${BIN}" >&2
  exit 1
fi

# 用 ASCII 标记检测；勿用 `strings | grep -q`（pipefail 下 grep 成功会 SIGPIPE 误报失败）
bcip_has_string() {
  LC_ALL=C grep -Fq "$1" < <(strings "$BIN")
}

ok=true
if ! bcip_has_string "synthesizing assistant output item"; then
  echo "警告: 未检测到流式修复标记（synthesizing assistant output item）。" >&2
  ok=false
fi
if ! bcip_has_string "patent/retriever.toml"; then
  echo "警告: 未检测到内置专利 Agent（patent/retriever.toml）。" >&2
  ok=false
fi

if $ok; then
  echo "OK: ${BIN}"
  echo "    流式修复: 已编入"
  echo "    内置专利 Agent: 已编入（retriever 等 9 角色）"
  echo "运行: ${BIN}"
else
  echo "请确认编译无 error；若仍失败可试: cargo clean -p codex-core -p codex-cli && 重跑本脚本" >&2
  exit 1
fi
