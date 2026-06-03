# BCIP IM 通道部署配置指南

## 配置文件: `~/.config/opencode/config.toml`

### 最小可用配置 (Telegram Only)

```toml
[im]
enabled = true

[im.telegram]
bot_token = "123456:ABC-DEF1234ghIkl-zyx57W2v1u123ew11"
allowed_users = [123456789]

[im.bridge]
server_url = "ws://127.0.0.1:3456"
max_reconnect = 10
heartbeat_interval_secs = 30
session_db_path = "~/.bcip/adapter-sessions.db"
```

### 完整配置 (Telegram + 飞书 + 钉钉)

```toml
[im]
enabled = true

# Telegram 适配器
[im.telegram]
bot_token = "BOT_TOKEN"
allowed_users = [USER_ID_1, USER_ID_2]

# 飞书适配器
[im.feishu]
app_id = "cli_xxxxxxxx"
app_secret = "xxxxxxxxxxxxxxxx"
allowed_users = ["user_id_1", "user_id_2"]

# 钉钉适配器
[im.dingtalk]
app_key = "dingxxxxxxxxx"
app_secret = "xxxxxxxxxxxxxxxx"
allowed_users = ["user_id_1"]

# Bridge 配置
[im.bridge]
server_url = "ws://127.0.0.1:3456"
max_reconnect = 10
heartbeat_interval_secs = 30
session_db_path = "~/.bcip/adapter-sessions.db"
```

## 环境变量覆盖

所有配置项可通过环境变量覆盖，优先级：环境变量 > config.toml > 默认值。

| 环境变量 | 对应配置 | 说明 |
|---------|---------|------|
| `BCIP_IM_ENABLED` | `im.enabled` | `"true"` / `"false"` |
| `BCIP_TELEGRAM_BOT_TOKEN` | `im.telegram.bot_token` | Telegram Bot API Token |
| `BCIP_FEISHU_APP_ID` | `im.feishu.app_id` | 飞书应用 App ID |
| `BCIP_FEISHU_APP_SECRET` | `im.feishu.app_secret` | 飞书应用 App Secret |
| `BCIP_DINGTALK_APP_KEY` | `im.dingtalk.app_key` | 钉钉应用 AppKey |
| `BCIP_DINGTALK_APP_SECRET` | `im.dingtalk.app_secret` | 钉钉应用 AppSecret |
| `BCIP_BRIDGE_SERVER_URL` | `im.bridge.server_url` | WebSocket Bridge 地址 |
| `BCIP_BRIDGE_DB_PATH` | `im.bridge.session_db_path` | 会话数据库路径 |

## 各平台配置步骤

### Telegram

1. 通过 @BotFather 创建 Bot，获取 `bot_token`
2. 获取你的 Telegram user_id（通过 @userinfobot）
3. 在 `config.toml` 中填入 `bot_token` 和 `allowed_users`
4. 启动 app-server，Bot 自动开始 polling

**Webhook 模式**（可选）：
- 需要 HTTPS 公网域名
- 设置 `BCIP_TELEGRAM_WEBHOOK_URL=https://your.domain.com/webhook/telegram`
- 自动从 polling 切换到 webhook

### 飞书

1. 在飞书开放平台创建企业自建应用
2. 开启机器人能力，获取 `app_id` 和 `app_secret`
3. 配置事件订阅 URL（如需 webhook 模式）
4. 在 `config.toml` 中填入配置

### 钉钉

1. 在钉钉开发者后台创建企业内部应用
2. 开启机器人能力，获取 `app_key` 和 `app_secret`
3. 配置 outgoing 回调 URL
4. 在 `config.toml` 中填入配置

## Bridge 架构

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│  Telegram    │     │   飞书       │     │   钉钉       │
│  Adapter     │     │  Adapter    │     │  Adapter    │
└──────┬───────┘     └──────┬──────┘     └──────┬──────┘
       │                    │                    │
       └────────┬───────────┘────────────────────┘
                │
        ┌───────▼───────┐
        │   ImBridge    │  WebSocket Client
        │  (Protocol    │  Session Store (SQLite)
        │   Adapter)    │  Heartbeat / Reconnect
        └───────┬───────┘
                │
        ┌───────▼───────┐
        │  BCIP Core    │  AgentBus (broadcast)
        │  AgentControl │  AgentControl (spawn)
        │  ToolRegistry │  ConstitutionalCheck
        └───────────────┘
```

## AgentBus 事件主题

| 主题 | 说明 | 发布者 |
|------|------|--------|
| `agent.lifecycle.spawned` | Agent 创建 | AgentControl |
| `agent.lifecycle.completed` | Agent 完成 | CompletionWatcher |
| `agent.communication.message` | 跨 Agent 消息 | AgentControl |
| `im.incoming.{platform}` | IM 入站消息 | ImBridge |
| `im.outgoing.{platform}` | IM 出站消息 | ImBridge |
| `patent.collaboration.{template}` | 协作工作流 | PatentWorkflow |

## 安全注意事项

1. **加密**: AgentBus 支持 AES-256-GCM 加密消息历史
   ```toml
   [im.bridge]
   encryption_key = "base64-encoded-32-byte-key"
   ```
   生成密钥: `openssl rand -base64 32`

2. **权限控制**: `allowed_users` 列表限制可访问的用户
3. **Session 存储**: SQLite 数据库路径应设置适当文件权限
4. **Bot Token**: 使用环境变量而非明文配置

## 故障排查

| 症状 | 检查项 |
|------|--------|
| Bot 无响应 | 检查 `BCIP_IM_ENABLED=true`、bot_token 正确 |
| 飞书 Token 失败 | 检查 app_id/app_secret、应用是否发布 |
| WebSocket 断连 | 检查 bridge.server_url、max_reconnect 设置 |
| 消息丢失 | 检查 AgentBus dead_letter_count() |
| 权限拒绝 | 检查 allowed_users 列表 |

## 编译 Feature Flags

```toml
# 启用所有专利工具 (含 IM)
cargo build --features patent-tools

# 仅 IM 适配器 (无专利工具)
cargo build -p codex-im-bridge -p codex-im-telegram -p codex-im-feishu -p codex-im-dingtalk

# 运行测试
cargo nextest run -p codex-im-common -p codex-im-bridge -p codex-im-telegram -p codex-im-feishu
```
