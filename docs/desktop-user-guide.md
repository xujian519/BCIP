# BCIP Desktop User Guide

## 安装

### macOS

1. 下载最新的 `BCIP.dmg`
2. 双击打开，将 BCIP 拖入 Applications
3. 首次打开需在「系统偏好设置 → 安全性与隐私」中允许

### Windows

1. 下载 `BCIP_x64-setup.exe`
2. 运行安装程序
3. 按照向导完成安装

## 首次配置

启动后需配置 LLM 服务端点:

1. 打开「设置」(Cmd/Ctrl + ,)
2. 填入 API 端点地址（如 `http://127.0.0.1:8788`）
3. 填入 API Key
4. 点击「连接测试」确认可用

配置文件位于 `~/.bcip/config.toml`。

## 核心功能

### 专利检索

- 在对话框中输入自然语言描述
- 系统自动调用 `PatentSearch` / `GooglePatents` 工具
- 结果包含专利号、标题、摘要、申请人

### 专利分析

- 上传专利文件（PDF/Word）
- 系统自动解析权利要求、技术特征
- 生成对比分析报告

### 专利撰写

- 输入技术方案描述
- 系统辅助生成权利要求书、说明书
- 内置 35 条专利法约束自动校验

### OA 答复

- 输入审查意见内容
- 系统分析对比文件、生成答复方案
- 支持多轮交互修改

## 快捷键

| 快捷键 | 功能 |
|--------|------|
| Cmd/Ctrl + N | 新会话 |
| Cmd/Ctrl + , | 设置 |
| Cmd/Ctrl + Enter | 发送消息 |
| Cmd/Ctrl + Shift + C | 复制最后回复 |

## 数据目录

- 配置: `~/.bcip/config.toml`
- 知识库: `~/.bcip/knowledge/`
- 下载: `~/.bcip/downloads/` 或 `$BCIP_DOWNLOAD_DIR`

## 常见问题

**Q: 连接 LLM 服务失败**
检查 `~/.bcip/config.toml` 中的 `model_provider` 和端点地址是否正确。

**Q: 知识库为空**
运行 `bcip knowledge refresh` 刷新知识索引。

**Q: 桌面端与 CLI 冲突**
BCIP 桌面端使用 `~/.bcip` 目录，与官方 Codex 桌面端 (`~/.codex`) 互不影响。

## 开发者

如需从源码构建桌面端:

```bash
cd apps/desktop
npm install
npm run tauri:build:bundle
```

详见 [DESKTOP_TESTING.md](../apps/desktop/docs/TESTING.md)
