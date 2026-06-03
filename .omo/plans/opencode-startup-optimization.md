# OpenCode 启动精简方案

> 目标：启动时间从 ~15-30s 降至 ~3-5s，消除 4/5 请求失败错误。

---

## 一、问题诊断

### 错误 1：`4 of 5 requests failed: config.providers, provider.list, app.agents, config.get`

**根因**：`opencode-antigravity-auth@1.6.0` 和 `opencode-openai-codex-auth` 两个插件在启动时向 Google/OpenAI 发起认证请求，但对应 API key 未配置或已过期，导致服务端初始化阻塞，第一批 TUI 请求超时。

**证据**：
- 智谱 API 本身可达（`curl` 返回 200）
- Google/OpenAI 的 provider 配置中没有 `apiKey`
- Explore 子 agent 也因 `GOOGLE_GENERATIVE_AI_API_KEY` 缺失而失败
- 错误来自 `/$bunfs/root/chunk-*.js`——即 Bun 运行时的 opencode 服务端

### 错误 2：启动慢（技能/命令过多）

**根因**：opencode 启动时扫描所有技能目录，每个技能注册为一个斜杠命令。当前共有 ~100+ 个技能：

| 来源 | 路径 | 数量 | 备注 |
|------|------|------|------|
| 用户技能 | `~/.claude/skills/` | 48 | 通用 AI 开发技能 |
| 专利技能 | `~/.agents/skills/` | 28 | 专利代理专用 |
| 配置技能 | `~/.config/opencode/skills/` | 2 | cli-anything, technical-deep-analysis |
| 备份技能 | `~/.config/opencode/skills/.backup_removed_skills/` | 19 | 已移除，不应加载但可能被扫描 |
| 插件注入 | superpowers | 14 | brainstorming, TDD, debugging 等 |
| **合计** | | **~111** | 每个都需要解析 SKILL.md + 注册命令 |

**重量级技能（体积大）**：
- `pdf` → 27MB
- `pptx` → 1.3MB
- `docx` → 1.3MB
- `patent-comparison` → 484KB
- `reasoning-sdk` → 464KB

---

## 二、技能精简方案

### 策略：三重分类

| 类别 | 说明 | 操作 |
|------|------|------|
| 🔴 **去重** | superpowers 插件已提供 | 从 `~/.claude/skills/` 删除 |
| 🟡 **归档** | 低频/实验性/非 BCIP | 移至 `~/.claude/skills/.archive/` |
| 🟢 **保留** | BCIP 开发高频使用 | 不变 |

### 2.1 去重列表（与 superpowers 插件重复）→ 删除

superpowers 插件（v5.1.0）已提供以下 14 个技能，`~/.claude/skills/` 中的同名副本可删除：

```
brainstorming/
dispatching-parallel-agents/
executing-plans/
finishing-a-development-branch/
receiving-code-review/
requesting-code-review/
subagent-driven-development/
systematic-debugging/
test-driven-development/
using-git-worktrees/
using-superpowers/
verification-before-completion/
writing-plans/
writing-skills/
```

### 2.2 归档列表（低频/非 BCIP）→ 移至 .archive/

| 技能 | 体积 | 归档理由 |
|------|------|----------|
| `pdf/` | 27MB | 专利场景用 markitdown 即可，pdf 太重 |
| `pptx/` | 1.3MB | 非日常开发需求 |
| `docx/` | 1.3MB | 非日常开发需求 |
| `reasoning-sdk/` | 464KB | 实验性，与 technical-deep-analysis 功能重叠 |
| `dual-reasoning/` | 308KB | 实验性 |
| `web-artifacts-builder/` | 52KB | 前端构建，BCIP 不需要 |
| `frontend-design/` | 20KB | 前端设计，BCIP 不需要 |
| `ollama-vision/` | 20KB | 本地视觉模型，日常不用 |
| `memory-sdk/` | 4KB | 实验性 |
| `playwright-cli/` | 64KB | 浏览器自动化，Rust 开发偶尔需要 |
| `tool-selector/` | 56KB | 实验性 |
| `swarm/` | 60KB | 并行 agent 调度，偶尔用 |
| `project-planning/` | 112KB | 项目规划，偶尔用 |
| `planning-workflow/` | 16KB | 与 writing-plans 重叠 |
| `dynamic-prompts/` | 4KB | 很少用 |
| `agent-templates/` | 12KB | 很少用 |
| `graphify/` | 60KB | 知识图谱，偶尔用 |
| `omc/` | 16KB | 多 agent 编排，偶尔用 |
| `style/` | 4KB | 输出样式切换，非必需 |
| `macos-calendar/` | 4KB | 非开发功能 |
| `keynote-cli/` | 24KB | 演示文稿，非日常 |
| `numbers-cli/` | 20KB | 电子表格，非日常 |
| `pages-cli/` | 20KB | 文档编辑，非日常 |
| `iterative-search/` | 52KB | 迭代搜索，偶尔用 |
| `ci-check/` | 8KB | 可手动运行 cargo 命令 |
| `personal-productivity/` | 8KB | 个人效率，非开发 |
| `baochen-finance/` | 188KB | 财务相关，非 BCIP |
| `academic-search/` | 36KB | ~/.agents/skills 中已有副本 |

### 2.3 保留列表（BCIP 核心 + 高频）

#### 从 ~/.claude/skills/ 保留（13 个）：

| 技能 | 体积 | 理由 |
|------|------|------|
| `documentation-lookup/` | 4KB | API 文档查询 |
| `markitdown/` | 4KB | 文档转换核心 |
| `libreoffice/` | 8KB | 批量文档转换 |
| `document-processor/` | 16KB | 统一文档处理入口 |
| `xlsx/` | 24KB | 数据分析 |
| `technical-deep-analysis/` | 52KB | 技术深度分析 |
| `web-access/` | 272KB | 网络访问 |
| `mermaid/` | 12KB | 图表绘制 |
| `omz/` | 8KB | Shell 效率 |
| `skill-creator/` | 60KB | 创建新技能时用 |
| `search-sdk/` | 4KB | 智能搜索 |
| `cli-anything/` | 272KB | CLI 构建工具 |

#### 从 ~/.agents/skills/ 保留（全部 28 个专利技能）：

这些都是 BCIP 核心业务技能，全部保留（部分去重）：
- 专利检索：patent-search, google-patents-search, patent-classification, patent-downloader, patent-search-local, patent-search-global
- 专利撰写：patent-drafting-v2, patent-ai, patent-comparison, patent-claim-extraction
- 专利审查：patent-guideline, patent-plan-mode, stop-patent-slop
- 法律智能：xiaona, storm-patent-experts, law-portrait, legal-qa
- 实务工具：cnipa-query, patent-archive-query, court-trip
- 通用：academic-search, document-processor, libreoffice, markitdown, search-sdk, personal-productivity, macos-calendar, keynote-cli, numbers-cli, pages-cli

### 2.4 启动时不应加载的目录

`~/.config/opencode/skills/.backup_removed_skills/` 包含 19 个已废弃技能，如果 opencode 扫描此目录则需要移到 skills 目录外。

---

## 三、插件精简方案

### 当前 7 个插件：

| 插件 | 必要性 | 建议 |
|------|--------|------|
| `superpowers@git+...` | 🔴 必需 | 提供 14 个开发流程技能 |
| `opencode-patent-plugin` | 🔴 必需 | BCIP 专利核心 |
| `oh-my-openagent` | 🟡 有用 | 提供 librarian/explore 等 agent，可保留但需修复 Google 模型配置 |
| `@tarquinen/opencode-dcp` | 🔴 必需 | 上下文压缩系统 |
| `cc-safety-net` | 🟢 保留 | 小体积，提供安全防护 |
| `opencode-antigravity-auth@1.6.0` | ❌ **应移除** | 无 Google API key，启动报错 |
| `opencode-openai-codex-auth` | ❌ **应移除** | 无 OpenAI API key，启动报错 |

### 建议操作：

1. **立即移除**：`opencode-antigravity-auth@1.6.0` 和 `opencode-openai-codex-auth`——这是 4/5 请求失败的根因
2. **修复**：`oh-my-openagent` 中 Google 模型的 agent 定义——当前 explore 等 agent 被配置使用 Google 模型但无 API key

---

## 四、MCP 服务精简

### 当前 5 个 MCP：

| MCP 服务 | 类型 | 启动影响 | 建议 |
|----------|------|----------|------|
| `web-reader` | 远程（智谱） | 低 | ✅ 保留 |
| `web-search-prime` | 远程（智谱） | 低 | ✅ 保留 |
| `zread` | 远程（智谱） | 低 | ✅ 保留（或与 web-reader 去重） |
| `codegraph` | 本地 Node | **中高** | ✅ 保留（代码导航核心） |
| `gemma4-multimodal` | 本地 Node | **中高** | 🟡 如不需图片分析可禁用 |

### 建议：
- `gemma4-multimodal` 如非必需可注释掉，减少一个本地进程启动等待
- `zread` 与 `web-reader` 功能可能重叠，确认后可选移除一个

---

## 五、操作计划

### Step 1：备份当前配置

```bash
cp ~/.config/opencode/opencode.json ~/.config/opencode/opencode.json.backup_$(date +%Y%m%d_%H%M%S)
```

### Step 2：移除报错插件

编辑 `~/.config/opencode/opencode.json`，从 `plugin` 数组中删除：
- `"opencode-antigravity-auth@1.6.0"`
- `"opencode-openai-codex-auth"`

### Step 2.5：启动前预检查（推荐）

```bash
# 确认当前 skills 列表，核对要归档的技能实际存在
ls ~/.claude/skills/
```

### Step 3：技能去重（归档 superpowers 重复项）

> 使用 `mv` 而非 `rm`，所有技能先移到 `.archive/` 而非删除，确保可恢复。

```bash
cd ~/.claude/skills/
mkdir -p .archive

# 将 superpowers 已提供的重复技能移至归档
for skill in brainstorming dispatching-parallel-agents executing-plans \
  finishing-a-development-branch receiving-code-review requesting-code-review \
  subagent-driven-development systematic-debugging test-driven-development \
  using-git-worktrees using-superpowers verification-before-completion \
  writing-plans writing-skills; do
  [ -d "$skill" ] && mv "$skill" .archive/
done
```

### Step 4：归档低频技能

```bash
cd ~/.claude/skills/

for skill in pdf pptx docx reasoning-sdk dual-reasoning \
  web-artifacts-builder frontend-design ollama-vision memory-sdk \
  playwright-cli tool-selector swarm project-planning planning-workflow \
  dynamic-prompts agent-templates graphify omc style \
  macos-calendar keynote-cli numbers-cli pages-cli \
  iterative-search ci-check personal-productivity baochen-finance \
  academic-search; do
  [ -d "$skill" ] && mv "$skill" .archive/
done
```

### Step 5：移动备份目录（防止误扫描）

```bash
# 将 backup_removed_skills 移到 skills 目录外
mv ~/.config/opencode/skills/.backup_removed_skills ~/.config/opencode/.backup_removed_skills
```

### Step 6：修复 oh-my-openagent 的 Google 模型配置

编辑 `~/.config/opencode/oh-my-openagent.json`，将使用 Google 模型的 agent 改为使用智谱模型：

```json
{
  "agents": {
    "librarian": { "model": "opencode/glm-4.7-free" },
    "explore": { "model": "deepseek/deepseek-chat" },
    "multimodal-looker": { "model": "deepseek/deepseek-chat" }
  }
}
```

### Step 7：可选：禁用 gemma4-multimodal MCP

如不需要本地图片分析，在 `opencode.json` 中将 `gemma4-multimodal` 的 `enabled` 改为 `false`。

### Step 8：验证

```bash
# 计时启动
time opencode run --help

# 完整启动测试
opencode run
```

---

## 六、预期效果

| 指标 | 精简前 | 精简后 |
|------|--------|--------|
| 技能数量 | ~111 个 | ~42 个（13 + 28 + 1） |
| 插件数量 | 7 个 | 5 个 |
| 磁盘占用（技能） | ~35MB | ~2MB |
| 启动错误 | 4/5 请求失败 | 0 错误 |
| 启动时间（估算） | 15-30s | 3-5s |

---

## 七、回滚方案

如精简后缺少某个技能，从 `.archive/` 恢复：

```bash
mv ~/.claude/skills/.archive/<skill名> ~/.claude/skills/
```

如需恢复完整配置，使用备份文件：

```bash
cp ~/.config/opencode/opencode.json.backup_YYYYMMDD_HHMMSS ~/.config/opencode/opencode.json
```
