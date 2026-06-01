<p align="center"><strong>YunPat Agent</strong> 云熙智能体 — 专利智能体
<p align="center">
  <img src="https://github.com/xujian519/BCIP/blob/main/.github/codex-cli-splash.png" alt="BCIP Agent splash" width="80%" />
</p>


---

## 项目简介

YunPat Agent 是一个基于 Rust 构建的知识产权全生命周期智能体平台，集成了专利检索、分析、撰写、审查、答复、无效和侵权分析等核心能力。

### 核心能力

| 能力域 | 说明 |
|--------|------|
| **专利检索** | Google Patents 搜索、IPC 分类、迭代检索、同义词扩展 |
| **技术分析** | 权利要求解析、对比分析、特征提取、语义比较 |
| **文件撰写** | 权利要求生成、说明书撰写、摘要生成 |
| **审查评估** | 新颖性评估、创造性评估、质量检查、形式审查 |
| **OA 答复** | 审查意见解析、策略推荐、答复生成 |
| **侵权/无效** | 侵权分析、无效分析、法律推理 |
| **合规治理** | 35 条宪法规则 + 6 个外部化规则，覆盖专利法 13 个阶段 |

---

## 架构概览

```
codex-rs/                          (Rust Workspace, 80+ crates)
├── codex-patent-core              领域类型与错误定义
├── codex-patent-domain            领域逻辑（20 个模块）
│   ├── claim_parser, compare, drafting
│   ├── oa, quality, disclosure, infringement
│   ├── rule_engine, legal_reasoning
│   └── examiner_simulator, guideline_graph
├── codex-patent-tools             50+ 注册工具（14 个模块）
│   ├── search, analysis, drafting, oa
│   ├── quality, evaluation, review
│   └── legal, management, document
├── codex-patent-agents            9 个 Agent 角色（TOML 配置驱动）
├── codex-patent-skills            12 个专利技能 + 8 个共享模块
├── codex-patent-knowledge         知识层
│   ├── SQLite 知识图谱 + 法律数据库
│   ├── 统一搜索 + 同义词字典
│   └── 知识卡片索引
├── codex-patent-constitutional    宪法规则引擎
├── codex-patent-text              CJK 分词器 + 相似度引擎 + IPC 分类器
├── codex-patent-scheduler         任务调度（Cron + 模板）
└── codex-patent-assets            静态资产
    ├── constitutional/            宪法规则 YAML
    ├── rules/                     外部化规则 YAML
    └── LLM Wiki 知识库            概念体系 + 专家提示词
```

### 9 个 Agent 角色

| 角色 | 职责 | 提示词深度 |
|------|------|-----------|
| Retriever | 专利检索专家 | 87 行 |
| Analyzer | 技术分析专家 | 99 行 |
| Writer | 文件撰写专家 | 112 行 |
| NoveltyChecker | 新颖性评估 | 107 行 |
| CreativityChecker | 创造性评估 | 133 行 |
| InfringementChecker | 侵权分析 | 106 行 |
| InvalidityChecker | 无效分析 | 110 行 |
| Reviewer | 文件审查 | 99 行 |
| QualityChecker | 质量评估 | 84 行 |

---

## Quickstart

### Installing and running BCIP Agent

Run the following on Mac or Linux to install BCIP Agent:

```shell
curl -fsSL # | sh
```

Run the following on Windows to install BCIP Agent:

```
powershell -ExecutionPolicy ByPass -c "irm # | iex"
```

BCIP Agent can also be installed via the following package managers:

```shell
# Install using npm
npm install -g @xujian519/bcip-agent
```

```shell
# Install using Homebrew
brew install --cask bcip-agent
```

Then simply run `bcip` to get started.

### Desktop app (Tauri + React)

源码在 [`apps/desktop/`](apps/desktop/)。在仓库根目录：

```bash
npm install --prefix apps/desktop   # 首次
npm run desktop:ci                  # lint + 构建 + generate-ts 校验
npm run desktop:e2e                 # Playwright E2E（需先 test:e2e:install）
npm run desktop:smoke               # 上述 + Tauri cargo check + app-server RPC
npm run tauri:dev                   # 本地桌面调试（Mac）
```

详见 [apps/desktop/README.md](apps/desktop/README.md)、[桌面端操作指南](DESKTOP_USER_GUIDE.md) 与 [桌面落地计划](docs/plans/2026-05-30-desktop-implementation-plan.md)。

<details>
<summary>You can also go to the <a href="https://github.com/xujian519/BCIP/releases/latest">latest GitHub Release</a> and download the appropriate binary for your platform.</summary>

Each GitHub Release contains many executables, but in practice, you likely want one of these:

- macOS
  - Apple Silicon/arm64: `bcip-agent-aarch64-apple-darwin.tar.gz`
  - x86_64 (older Mac hardware): `bcip-agent-x86_64-apple-darwin.tar.gz`
- Linux
  - x86_64: `bcip-agent-x86_64-unknown-linux-musl.tar.gz`
  - arm64: `bcip-agent-aarch64-unknown-linux-musl.tar.gz`

</details>

### Using BCIP Agent with your ChatGPT plan

Run `bcip` and select **Sign in with ChatGPT**. We recommend signing into your ChatGPT account to use BCIP Agent as part of your Plus, Pro, Business, Edu, or Enterprise plan.

You can also use BCIP Agent with an API key, but this requires [additional setup](#).

---

## Patent System Documentation

- [**专利系统四维映射**](./docs/patent-system-map.md) — Agent ↔ Skill ↔ Tool ↔ Concept ↔ ConstitutionalRule
- [**系统审阅报告**](./docs/comprehensive-review-report.md) — 当前系统状态与差距分析
- [**实施计划**](./docs/implementation-plan.md) — Phase 1-6 增强路线图
- [**Phase 1 检查清单**](./docs/phase1-agent-checklist.md) — Agent 提示词重写（已完成）
- [**Phase 3-6 检查清单**](./docs/phase3-6-checklist.md) — 清理修复/工具增强/规则连接/映射
- [**yunpat-agent 集成分析**](./docs/deep-analysis-yunpat-to-bcip.md) — yunpat → BCIP 资产移植分析

## General Docs

- [**BCIP Agent Documentation**](#)
- [**Contributing**](./docs/contributing.md)
- [**Installing & building**](./docs/install.md)
- [**Getting started**](./docs/getting-started.md)
- [**Configuration**](./docs/config.md)
- [**Sandbox**](./docs/sandbox.md)
- [**Open source fund**](./docs/open-source-fund.md)

---

## Tech Stack

| 维度 | 技术 |
|------|------|
| 语言 | Rust (1.7M+ LOC), TypeScript |
| TUI | [Ratatui](https://ratatui.rs/) |
| 构建 | Cargo + Bazel, `just` |
| 知识库 | SQLite (laws.db + patent_kg.db) |
| 规则引擎 | YAML 配置化，运行时动态加载 |
| AI 接入 | OpenAI API, DeepSeek, Ollama, LM Studio |
| 扩展机制 | MCP Server/Client, Plugins, Extensions, Skills |

This repository is licensed under the [Apache-2.0 License](LICENSE).
