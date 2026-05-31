# BCIP CI/CD Strategy

## Workflows

### `bcip-ci.yml` — Main CI Pipeline

触发条件：所有 PR + push to `main`

| Job | 说明 |
|-----|------|
| `detect` | 检测变更路径，按类型分流 |
| `rust-fmt` | `cargo fmt --check` |
| `rust-clippy` | Clippy 检查（专利模块 + IM 模块 + API） |
| `rust-test` | Nextest 测试（专利模块 + IM 模块） |
| `rust-build` | 编译 `codex-cli` release 二进制 |
| `python-lint` | Ruff lint + format check |
| `python-test` | pytest |
| `security` | `cargo deny` 安全扫描 |

### `bcip-release.yml` — Release Pipeline

触发条件：tag push (`v*`) 或手动触发

| Job | 说明 |
|-----|------|
| `build-linux` | Linux x86_64 + aarch64 构建 |
| `build-macos` | macOS ARM64 + x86_64 构建 |
| `release` | 创建 GitHub Release + 上传产物 + SHA256 校验 |

### `ci.yml` — 上游 CI（已禁用）

保留用于上游合并参考，仅 `workflow_dispatch` 触发。

## 上游 Workflow

上游 OpenAI Codex 的 workflow 文件归档在 `_upstream/` 目录中，不会自动触发。
