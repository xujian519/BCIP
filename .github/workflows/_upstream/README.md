# 上游 Workflow 归档

此目录包含从上游 OpenAI Codex 继承的 workflow 文件，已归档不会触发。

BCIP 使用以下 CI/CD workflow：
- `bcip-ci.yml` — 主 CI 流水线（PR + push to main）
- `bcip-release.yml` — 发布流水线（tag push 触发）

如需恢复某个 workflow，将其移回上级目录并调整触发条件即可。
