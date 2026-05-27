# BCIP 品牌统一实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将仓库中所有用户可见的 "OpenAI Codex"、"Codex" 品牌名统一为 "BCIP Agent" / "BCIP"，外部链接删除或替换为占位符。

**Architecture:** 分 4 个阶段执行，每个阶段独立可测试、可回滚。范围限定为用户可见文本，不涉及内部代码标识符（目录名、crate名、变量名等）。

**Tech Stack:** Rust (TUI/CLI)、TypeScript (SDK)、Python (SDK)、Markdown (文档)

---

## 品牌映射规则

| 原始文本 | 替换为 | 适用场景 |
|----------|--------|---------|
| `OpenAI Codex` | `BCIP Agent` | 标题栏、启动横幅、欢迎页 |
| `OpenAI's command-line coding agent` | `an intelligent coding agent` | 欢迎页描述 |
| `OpenAI Curated` | `BCIP Curated` | 插件市场标签 |
| `OpenAI API key` | `API key` | 认证界面 |
| `developers.openai.com/codex/*` | 删除链接或替换为 `#`（占位符） | 文档外部链接 |
| `chatgpt.com/codex/*` | 删除链接或替换为 `#` | 安装/产品页面链接 |
| `Ask Codex to` / `tell Codex` | `Ask BCIP to` / `tell BCIP` | 聊天占位符、提示文本 |
| `Codex can` / `Codex uses` | `BCIP can` / `BCIP uses` | 功能描述 |
| `class Codex` (TypeScript/Python) | `class BCIPAgent`（保留 Codex 别名） | SDK 主类 |
| `openai_codex/` (Python 包路径) | `bcip_agent/` | Python 包目录 |
| `"codex-monorepo"` | `"bcip-monorepo"` | 顶级 package.json |

**不修改：** 数据目录 `~/.codex/`、内部 crate 名、目录名 `codex-rs/`、内部变量名、环境变量名。

---

## 阶段 1: TUI 界面文本（风险最低）

### Task 1.1: 替换状态栏和历史会话中的品牌名

**Files:**
- Modify: `codex-rs/tui/src/status/card.rs:688`
- Modify: `codex-rs/tui/src/history_cell/session.rs:340-343, 410`

- [ ] **Step 1: 修改 status/card.rs**

将第 688 行的 `Span::from("OpenAI Codex").bold()` 替换为 `Span::from("BCIP Agent").bold()`

- [ ] **Step 2: 修改 history_cell/session.rs**

将第 343 行 `Span::from("OpenAI Codex").bold()` 替换为 `Span::from("BCIP Agent").bold()`
将第 410 行 `format!("OpenAI Codex (v{})` 替换为 `format!("BCIP Agent (v{})`

- [ ] **Step 3: 运行测试验证**

```bash
cd codex-rs && just test -p codex-tui
```

- [ ] **Step 4: 更新受影响的 snapshot 文件**

```bash
cd codex-rs && cargo insta pending-snapshots -p codex-tui
cargo insta accept -p codex-tui
```

- [ ] **Step 5: Commit**

```bash
git add -A && git commit -m "fix(tui): 替换状态栏和历史会话中的 OpenAI Codex 为 BCIP Agent"
```

---

### Task 1.2: 替换欢迎页和认证界面的品牌名

**Files:**
- Modify: `codex-rs/tui/src/onboarding/welcome.rs:97-98`
- Modify: `codex-rs/tui/src/onboarding/auth.rs:651`

- [ ] **Step 1: 修改 welcome.rs**

将第 97 行 `"Codex".bold()` 替换为 `"BCIP Agent".bold()`
将第 98 行 `", OpenAI's command-line coding agent".into()` 替换为 `", an intelligent coding agent".into()`

- [ ] **Step 2: 修改 auth.rs**

将第 651 行 `"Use your own OpenAI API key for usage-based billing"` 替换为 `"Use your own API key for usage-based billing"`

- [ ] **Step 3: 运行测试并更新 snapshot**

```bash
cd codex-rs && just test -p codex-tui && cargo insta accept -p codex-tui
```

- [ ] **Step 4: Commit**

```bash
git add -A && git commit -m "fix(tui): 统一欢迎页和认证界面的品牌名为 BCIP Agent"
```

---

### Task 1.3: 替换 exec 模块启动横幅

**Files:**
- Modify: `codex-rs/exec/src/event_processor_with_human_output.rs:219`
- Modify: `codex-rs/exec/src/event_processor_with_human_output.rs:103, 412`

- [ ] **Step 1: 修改启动横幅**

将第 219 行 `eprintln!("OpenAI Codex v{VERSION}\n--------")` 替换为 `eprintln!("BCIP Agent v{VERSION}\n--------")`

- [ ] **Step 2: 修改代理名称显示**

将第 103 行和第 412 行中 `"codex".style(...)` 的代理名称替换为 `"bcip".style(...)`

- [ ] **Step 3: 运行测试**

```bash
cd codex-rs && just test -p codex-exec
```

- [ ] **Step 4: Commit**

```bash
git add -A && git commit -m "fix(exec): 替换启动横幅和代理名称为 BCIP Agent"
```

---

### Task 1.4: 替换插件市场的品牌名

**Files:**
- Modify: `codex-rs/tui/src/chatwidget/plugins.rs:1496-1505`
- Modify: `codex-rs/tui/src/chatwidget/model_popups.rs:48`

- [ ] **Step 1: 修改 plugins.rs**

将所有 `OpenAI Curated` 替换为 `BCIP Curated`：
- 第 1496 行: `"OpenAI Curated"` → `"BCIP Curated"`
- 第 1498 行: `"OpenAI Curated marketplace."` → `"BCIP Curated marketplace."`
- 第 1499 行: `format!("Installed {curated_installed} of {curated_total} OpenAI Curated plugins.")` → 同样替换
- 第 1504-1505 行: 同上

- [ ] **Step 2: 修改 model_popups.rs**

将第 48 行 `"Warning: OpenAI base URL is overridden to {base_url}."` 替换为 `"Warning: API base URL is overridden to {base_url}."`

- [ ] **Step 3: 运行测试并更新 snapshot**

```bash
cd codex-rs && just test -p codex-tui && cargo insta accept -p codex-tui
```

- [ ] **Step 4: Commit**

```bash
git add -A && git commit -m "fix(tui): 替换插件市场和模型弹窗中的 OpenAI 品牌名"
```

---

### Task 1.5: 替换宠物系统中的品牌名

**Files:**
- Modify: `codex-rs/tui/src/pets/catalog.rs:21`
- Modify: `codex-rs/tui/src/pets/picker.rs:256`

- [ ] **Step 1: 修改 catalog.rs**

将第 21 行 `display_name: "Codex"` 替换为 `display_name: "BCIP Agent"`

- [ ] **Step 2: 修改 picker.rs**

将第 256 行 `"Codex"` 替换为 `"BCIP Agent"`
将第 282 行 `assert_eq!(params.items[2].name, "Codex")` 替换为 `assert_eq!(params.items[2].name, "BCIP Agent")`

- [ ] **Step 3: 运行测试并更新 snapshot**

```bash
cd codex-rs && just test -p codex-tui && cargo insta accept -p codex-tui
```

- [ ] **Step 4: Commit**

```bash
git add -A && git commit -m "fix(tui): 替换宠物系统中的品牌名"
```

---

### Task 1.6: 批量更新所有 snapshot 文件中的品牌引用

**Files:**
- All `*.snap` files in `codex-rs/tui/src/` subdirectories (~60+ files)
- All `*.snap` files in `codex-rs/tui/src/chatwidget/snapshots/`
- All `*.snap` files in `codex-rs/tui/src/bottom_pane/snapshots/`
- All `*.snap` files in `codex-rs/tui/src/snapshots/`
- All `*.snap` files in `codex-rs/tui/src/history_cell/snapshots/`
- All `*.snap` files in `codex-rs/tui/src/status/snapshots/`

- [ ] **Step 1: 使用 sed 批量替换 snapshot 文件**

注意：snapshot 文件由测试自动生成。正确做法是先修改源代码中的用户可见字符串，然后运行测试让 snapshot 自动更新。因此此 Task 应在 Task 1.1-1.5 完成后执行。

首先搜索所有源文件中剩余的用户可见 "Codex" 字符串（在 TUI 的 .rs 文件中）：

```bash
rg '"Codex' codex-rs/tui/src/ --type rust -l
rg 'Ask Codex' codex-rs/tui/src/ --type rust -l
rg 'tell Codex' codex-rs/tui/src/ --type rust -l
rg 'Codex can' codex-rs/tui/src/ --type rust -l
```

逐文件修改这些字符串（参考品牌映射规则）。

- [ ] **Step 2: 运行完整 TUI 测试套件**

```bash
cd codex-rs && just test -p codex-tui
```

- [ ] **Step 3: 检查并接受所有 snapshot 更新**

```bash
cd codex-rs && cargo insta pending-snapshots -p codex-tui
cargo insta accept -p codex-tui
```

- [ ] **Step 4: Commit**

```bash
git add -A && git commit -m "fix(tui): 批量更新所有 snapshot 中的品牌名为 BCIP Agent"
```

---

## 阶段 2: CLI 帮助文本

### Task 2.1: 统一 main.rs 中的品牌名混用

**Files:**
- Modify: `codex-rs/cli/src/main.rs` 多处

- [ ] **Step 1: 替换用户可见消息中的混用品牌名**

修改以下行（将 Codex 统一为 BCIP Agent）：
- 第 706 行: `"Please restart Codex."` → `"Please restart BCIP Agent."`
- 第 714 行: 已使用 BCIP，确认一致
- 第 388 行: `"printenv OPENAI_API_KEY | codex login --with-api-key"` → `"printenv OPENAI_API_KEY | bcip login --with-api-key"`
- 第 394 行: `"printenv CODEX_ACCESS_TOKEN | codex login --with-access-token"` → `"printenv BCIP_ACCESS_TOKEN | bcip login --with-access-token"`
- 第 1173 行: 同上替换
- 第 1464 行: `` "`--profile only applies to runtime commands and `codex mcp`" `` → 使用 `bcip mcp`
- 第 1533 行: `` "run `codex login` first" `` → `` "run `bcip login` first" ``
- 第 1797 行: `` "`codex {subcommand}`" `` → `` "`bcip {subcommand}`" `` (3处)
- 第 1802 行: 同上
- 第 1890 行: 同上
- 第 1282 行: `` "`codex sandbox` is not supported" `` → `` "`bcip sandbox` is not supported" ``
- 第 1984 行: `"Codex's interactive TUI"` → `"BCIP Agent's interactive TUI"`
- 第 2642, 2670 行: `"run codex resume"` → `"run bcip resume"`

- [ ] **Step 2: 替换 override_usage 中的命令名**

第 97 行: `override_usage = "codex [OPTIONS]..."` → `"bcip [OPTIONS]..."`
第 96 行: `bin_name = "codex"` → `bin_name = "bcip"`
第 2165 行: `let name = "codex"` → `let name = "bcip"`

- [ ] **Step 3: 运行测试**

```bash
cd codex-rs && just test -p codex-cli
```

- [ ] **Step 4: Commit**

```bash
git add -A && git commit -m "fix(cli): 统一 main.rs 中用户可见的品牌名和命令名为 BCIP/bcip"
```

---

### Task 2.2: 统一 exec 模块的 CLI 文本

**Files:**
- Modify: `codex-rs/exec/src/cli.rs:12`
- Modify: `codex-rs/exec/src/lib.rs:316`

- [ ] **Step 1: 修改 exec cli.rs**

第 12 行: `override_usage = "codex exec [OPTIONS]..."` → `"bcip exec [OPTIONS]..."`

- [ ] **Step 2: 修改 exec lib.rs**

第 316 行: `"Error finding codex home: {err}"` → `"Error finding BCIP home: {err}"`

- [ ] **Step 3: 运行测试**

```bash
cd codex-rs && just test -p codex-exec
```

- [ ] **Step 4: Commit**

```bash
git add -A && git commit -m "fix(exec): 统一 CLI 用法字符串和错误消息中的品牌名"
```

---

### Task 2.3: 统一 mcp_cmd.rs 中的命令引用

**Files:**
- Modify: `codex-rs/cli/src/mcp_cmd.rs:78, 400, 608, 948`

- [ ] **Step 1: 替换所有 `codex mcp` 引用为 `bcip mcp`**

- 第 78 行: `override_usage = "codex mcp add ..."` → `"bcip mcp add ..."`
- 第 400 行: `` "Run `codex mcp login {name}`" `` → `` "Run `bcip mcp login {name}`" ``
- 第 608 行: `` "Try `codex mcp add my-tool -- my-command`" `` → `` "Try `bcip mcp add my-tool -- my-command`" ``
- 第 948 行: `"remove: codex mcp remove {}"` → `"remove: bcip mcp remove {}"`

- [ ] **Step 2: 运行测试**

```bash
cd codex-rs && just test -p codex-cli
```

- [ ] **Step 3: Commit**

```bash
git add -A && git commit -m "fix(cli): 统一 mcp 命令中的品牌名为 bcip"
```

---

## 阶段 3: SDK 类名与包路径

### Task 3.1: TypeScript SDK - 重命名主类并添加别名

**Files:**
- Modify: `sdk/typescript/src/codex.ts`
- Modify: `sdk/typescript/src/index.ts`
- Modify: `sdk/typescript/src/exec.ts`
- Modify: `sdk/typescript/src/codexOptions.ts`
- Modify: `sdk/typescript/src/thread.ts`

- [ ] **Step 1: 在 codex.ts 中添加别名导出**

在第 11 行 `export class Codex {` 后面添加：
```typescript
/** @deprecated Use BCIPAgent instead. Alias kept for backward compatibility. */
export const BCIPAgent = Codex;
```

同时在 index.ts 中添加：
```typescript
export { Codex, BCIPAgent } from "./codex";
```

- [ ] **Step 2: 运行 TypeScript SDK 测试**

```bash
cd sdk/typescript && npm test
```

- [ ] **Step 3: Commit**

```bash
git add -A && git commit -m "feat(sdk-ts): 导出 BCIPAgent 别名，保留 Codex 兼容"
```

---

### Task 3.2: Python SDK - 重命名包路径和主类

**Files:**
- Move: `sdk/python/src/openai_codex/` → `sdk/python/src/bcip_agent/`
- Modify: `sdk/python/pyproject.toml:45`
- Modify: `sdk/python/src/bcip_agent/__init__.py`
- Modify: `sdk/python/src/bcip_agent/api.py`

- [ ] **Step 1: 重命名包目录**

```bash
mv sdk/python/src/openai_codex sdk/python/src/bcip_agent
```

- [ ] **Step 2: 更新 pyproject.toml**

将第 45 行 `packages = ["src/openai_codex"]` 替换为 `packages = ["src/bcip_agent"]`

- [ ] **Step 3: 在 api.py 中添加 BCIPAgent 别名**

在 `class Codex` 定义后添加：
```python
BCIPAgent = Codex  # backward-compatible alias
```

- [ ] **Step 4: 更新 __init__.py 导出**

确保 `__init__.py` 同时导出 `Codex` 和 `BCIPAgent`：
```python
from .api import Codex, AsyncCodex, BCIPAgent
```

- [ ] **Step 5: 更新内部导入**

在所有 `bcip_agent/` 目录下的文件中，将 `from openai_codex.` 替换为 `from bcip_agent.`

- [ ] **Step 6: 运行 Python SDK 测试**

```bash
cd sdk/python && python -m pytest
```

- [ ] **Step 7: Commit**

```bash
git add -A && git commit -m "feat(sdk-py): 重命名包路径为 bcip_agent，添加 BCIPAgent 别名"
```

---

### Task 3.3: SDK 示例和测试中的品牌引用更新

**Files:**
- Modify: `sdk/typescript/samples/basic_streaming.ts:10`
- Modify: `sdk/typescript/samples/structured_output.ts:7`
- Modify: `sdk/typescript/samples/structured_output_zod.ts:8`
- Modify: `sdk/typescript/samples/helpers.ts:3, 6`
- Modify: `sdk/typescript/tests/testCodex.ts` 多处

- [ ] **Step 1: 更新 TypeScript 示例**

示例文件中的 `new Codex(...)` 保持不变（Codex 类名保留），但注释和描述中引用品牌的文本需要统一。

- [ ] **Step 2: 运行测试验证**

```bash
cd sdk/typescript && npm test
```

- [ ] **Step 3: Commit**

```bash
git add -A && git commit -m "fix(sdk-ts): 更新示例和测试中的品牌引用"
```

---

## 阶段 4: 文档与 CI 用户可见文本

### Task 4.1: 更新顶级 package.json

**Files:**
- Modify: `package.json:2`

- [ ] **Step 1: 修改 name 字段**

将第 2 行 `"name": "codex-monorepo"` 替换为 `"name": "bcip-monorepo"`

- [ ] **Step 2: Commit**

```bash
git add package.json && git commit -m "chore: 重命名 monorepo 为 bcip-monorepo"
```

---

### Task 4.2: 清理 README.md 中的外部链接

**Files:**
- Modify: `README.md`

- [ ] **Step 1: 替换外部链接为占位符**

将所有 `developers.openai.com/codex/*` 链接替换为 `#` 并添加注释标记：
- 第 6 行: `https://developers.openai.com/codex/ide` → `#`（待自有文档上线后更新）
- 第 62 行: `https://developers.openai.com/codex/auth#sign-in-with-an-api-key` → `#`
- 第 66 行: `https://developers.openai.com/codex` → `#`

将所有 `chatgpt.com/codex/*` 链接替换为 `#`：
- 第 7 行: `https://chatgpt.com/codex?app-landing-page=true` → `#`
- 第 8 行: `https://chatgpt.com/codex` → `#`
- 第 19 行: `https://chatgpt.com/codex/install.sh` → `#`
- 第 25 行: `https://chatgpt.com/codex/install.ps1` → `#`
- 第 60 行: `https://help.openai.com/en/articles/11369540-codex-in-chatgpt` → `#`

- [ ] **Step 2: Commit**

```bash
git add README.md && git commit -m "docs: 清理 README 中的外部链接为占位符"
```

---

### Task 4.3: 清理 docs/ 目录下的外部链接

**Files:**
- Modify: `docs/slash_commands.md:3`
- Modify: `docs/exec.md:3`
- Modify: `docs/skills.md:3`
- Modify: `docs/config.md:3, 5, 7`
- Modify: `docs/getting-started.md:3`
- Modify: `docs/execpolicy.md:3`
- Modify: `docs/example-config.md:3`
- Modify: `docs/sandbox.md:3`
- Modify: `docs/agents_md.md:3`
- Modify: `docs/authentication.md:3`
- Modify: `docs/open-source-fund.md:8`

- [ ] **Step 1: 批量替换 docs/ 下的外部链接**

所有 `https://developers.openai.com/codex/*` 链接替换为 `#`（保留链接文本，仅替换 URL）。

`docs/open-source-fund.md` 第 8 行 `https://openai.com/form/codex-open-source-fund/` → `#`

- [ ] **Step 2: Commit**

```bash
git add docs/ && git commit -m "docs: 清理所有文档中的外部链接为占位符"
```

---

### Task 4.4: 更新 SECURITY.md

**Files:**
- Modify: `SECURITY.md`

- [ ] **Step 1: 替换外部链接和品牌引用**

- 第 7 行: `"The security is essential to OpenAI's mission."` → `"Security is essential to BCIP's mission."`
- 第 17 行: `https://developers.openai.com/codex/agent-approvals-security` → `#`
- Bugcrowd 链接（第 9、13 行）根据实际情况决定是否保留

- [ ] **Step 2: Commit**

```bash
git add SECURITY.md && git commit -m "docs(security): 更新品牌名和外部链接"
```

---

### Task 4.5: 更新 GitHub Issue 模板

**Files:**
- Modify: `.github/ISSUE_TEMPLATE/1-codex-app.yml`
- Modify: `.github/ISSUE_TEMPLATE/6-docs-issue.yml`
- Modify: `.github/ISSUE_TEMPLATE/4-bug-report.yml`

- [ ] **Step 1: 检查并修复模板中的品牌引用**

- `1-codex-app.yml`: 已使用 BCIP，确认一致
- `6-docs-issue.yml` 第 8 行: `"It helps make Codex better."` → `"It helps make BCIP Agent better."`
- `4-bug-report.yml` 第 12 行: 确认已使用 BCIP

- [ ] **Step 2: Commit**

```bash
git add .github/ISSUE_TEMPLATE/ && git commit -m "fix(github): 统一 Issue 模板中的品牌名"
```

---

### Task 4.6: 更新 GitHub Workflow 中的用户可见文本

**Files:**
- Modify: `.github/workflows/rust-release.yml:521, 794, 1105`
- Modify: `.github/workflows/ci.yml:29`
- Modify: `.github/actions/setup-rusty-v8/action.yml:2`
- Modify: `.github/actions/windows-code-sign/action.yml:2`
- Modify: `.github/actions/macos-code-sign/action.yml:2`

- [ ] **Step 1: 更新 job name**

- `rust-release.yml` 第 521 行: `name: Build Codex package archive` → `name: Build BCIP package archive`
- `rust-release.yml` 第 794 行: 同上
- `rust-release.yml` 第 1105 行: `name: Add Codex package checksum manifest` → `name: Add BCIP package checksum manifest`
- `ci.yml` 第 29 行: `name: Test Codex package builder` → `name: Test BCIP package builder`

- [ ] **Step 2: 更新 action description**

- `setup-rusty-v8/action.yml` 第 2 行: `"Download and verify Codex-built rusty_v8 artifacts"` → `"Download and verify BCIP-built rusty_v8 artifacts"`
- `windows-code-sign/action.yml` 第 2 行: 确认描述
- `macos-code-sign/action.yml` 第 2 行: 确认描述

- [ ] **Step 3: Commit**

```bash
git add .github/ && git commit -m "fix(ci): 统一 GitHub Workflow 中的用户可见品牌名"
```

---

### Task 4.7: 更新 codex-rs/README.md 和 app-server/README.md

**Files:**
- Modify: `codex-rs/README.md:88`
- Modify: `codex-rs/app-server/README.md` 多处

- [ ] **Step 1: 更新 codex-rs/README.md**

第 88 行: `~/.codex/memories` → 保持不变（数据目录不迁移）

确认其余内容已使用 BCIP Agent。

- [ ] **Step 2: 更新 app-server/README.md 中的品牌引用**

- 第 1 行: `# codex-app-server` → `# BCIP App Server`
- 第 91 行: `OpenAI Compliance Logs Platform` → 保持不变（技术描述）
- 第 1794 行: `https://auth.openai.com/codex/device` → `#`

检查其余 OpenAI/Codex 混用位置并统一。

- [ ] **Step 3: Commit**

```bash
git add codex-rs/README.md codex-rs/app-server/README.md && git commit -m "docs: 统一 app-server README 中的品牌名"
```

---

### Task 4.8: 更新 debug_config.rs 中的路径

**Files:**
- Modify: `codex-rs/tui/src/debug_config.rs:628`

- [ ] **Step 1: 更新 Windows 路径中的品牌名**

第 628 行: `absolute_path("C:\\ProgramData\\OpenAI\\Codex\\requirements.toml")` → `absolute_path("C:\\ProgramData\\BCIP\\requirements.toml")`

注意：此路径用于调试配置示例，实际运行路径取决于安装配置，可能不需要修改。需确认。

- [ ] **Step 2: Commit**

```bash
git add codex-rs/tui/src/debug_config.rs && git commit -m "fix(tui): 更新调试配置中的品牌路径"
```

---

## 最终验证

### Task 5.1: 全量品牌名扫描验证

- [ ] **Step 1: 搜索残留的 "OpenAI Codex"**

```bash
rg "OpenAI Codex" --type rust --type ts --type py --type md
```

预期：0 匹配

- [ ] **Step 2: 搜索残留的 developers.openai.com 链接**

```bash
rg "developers\.openai\.com" --type md
```

预期：0 匹配

- [ ] **Step 3: 搜索残留的 chatgpt.com/codex 链接**

```bash
rg "chatgpt\.com/codex" --type md
```

预期：0 匹配

- [ ] **Step 4: 运行完整测试套件**

```bash
cd codex-rs && just test -p codex-tui && just test -p codex-cli && just test -p codex-exec
```

- [ ] **Step 5: 生成验证报告**

将上述搜索结果汇总，确认所有用户可见的品牌名已统一为 BCIP Agent / BCIP。

---

## 风险与回滚

| 风险 | 应对措施 |
|------|---------|
| Snapshot 测试大量更新 | 每个阶段单独运行测试并接受 snapshot |
| Python 包重命名破坏导入 | 保留 `openai_codex` 兼容层，添加 re-export |
| TypeScript 类名重命名破坏下游 | 使用别名导出，不删除原名 |
| CI workflow 修改影响发布流程 | 仅修改 name/description，不改 artifact 路径 |
| 某些 "OpenAI" 引用是技术描述而非品牌 | 仔细区分，仅替换品牌引用 |

**回滚策略：** 每个阶段独立 commit，可按阶段 revert。
