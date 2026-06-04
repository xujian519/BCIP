# Configuration

BCIP 使用 `config.toml` 进行配置，默认位于 `~/.bcip/config.toml`。

配置文件可通过环境变量 `BCIP_HOME`（或 `CODEX_HOME`）指定替代目录。

## 基本配置

```toml
# config.toml

# LLM 服务端点（默认 http://127.0.0.1:8788）
model_provider = "openai-compatible"
model_provider_id = "bcip-local"

# 模型选择
model = "default"

# 审批策略: "suggest" | "auto-edit" | "full-auto"
approval_policy = "suggest"

# 工作目录
cwd = "/path/to/project"
```

## 完整配置参考

### 顶层字段

| 字段 | 类型 | 说明 |
|------|------|------|
| `model_provider` | string | LLM 服务提供商标识 |
| `model_provider_id` | string | 提供商实例 ID |
| `model` | string | 默认模型名称 |
| `approval_policy` | string | 工具审批策略 |
| `cwd` | string | 默认工作目录 |
| `features` | object | 功能特性开关 |

### Agent 配置 (`[agents]`)

```toml
[agents]
# 最大并发 Agent 线程数（不设置则无限制）
max_threads = 10
# 最大嵌套深度（默认由系统决定）
max_depth = 3
# 任务最大运行时间（秒）
job_max_runtime_seconds = 300
# 中断时是否记录消息
interrupt_message = true

# 自定义 Agent 角色
[agents.patent_searcher]
description = "专利检索专家"
nickname_candidates = ["searcher", "检索员"]
config_file = "agents/patent_searcher.toml"
```

### Analytics 配置 (`[analytics]`)

```toml
[analytics]
enabled = false
```

### TUI 配置 (`[tui]`)

```toml
[tui]
# 备用屏幕模式: "auto" | "always" | "never"
alt_screen_mode = "auto"
```

## 环境变量

| 变量 | 说明 | 默认值 |
|------|------|--------|
| `BCIP_HOME` | 用户数据目录 | `~/.bcip` |
| `BCIP_MLX_URL` | MLX 嵌入服务地址 | `http://localhost:8009` |
| `BCIP_MLX_API_KEY` | MLX 嵌入服务 API Key | - |
| `BCIP_MLX_MODEL` | MLX 嵌入模型名称 | `bge-m3-mlx-8bit` |
| `BCIP_DOWNLOAD_DIR` | 专利 PDF 下载目录 | 系统临时目录 |

## Lifecycle Hooks

管理员可在 `requirements.toml` 中设置 `allow_managed_hooks_only = true`，忽略用户/项目/会话级别的 hook 配置，仅保留托管 hook。

完整 JSON Schema 参考: `codex-rs/core/config.schema.json`
