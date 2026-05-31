# LiteLLM / 8788 代理：不重新编译 BCIP 的变通

BCIP TUI 崩溃原因：代理只发 `response.output_text.delta`，没有先发 `response.output_item.added`。

在 **8788 的 Responses SSE 流**里，第一个 text delta 之前插入一条（与 OpenAI 兼容）：

```json
{"type":"response.output_item.added","item":{"type":"message","role":"assistant","content":[]}}
```

流结束、`response.completed` 之前若从未发过 `output_item.done`，可补：

```json
{"type":"response.output_item.done","item":{"type":"message","role":"assistant","content":[]}}
```

改完代理后，**无需重编** `bcip`，旧二进制即可正常对话。

Rust 侧修复在 `codex-api` 的 SSE normalizer；本地验证用 `scripts/bcip-dev-build.sh`。
