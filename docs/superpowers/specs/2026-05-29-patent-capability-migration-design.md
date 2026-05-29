# 专利能力迁移设计文档

**日期**: 2026-05-29
**目标**: 将 YunXi (云熙智能体) 的全部专利能力深度集成到 BCIP (宝宸知识产权)

---

## 1. 概述

### 1.1 源项目: YunXi

YunXi 是一款专业专利智能体，拥有 50+ 专利工具、9 个专利 Agent 角色、SQLite 知识图谱 (40K 节点)、法律法规库、150+ 专利知识文档等完整专利能力栈。

### 1.2 目标项目: BCIP

BCIP (宝宸知识产权) 当前是基于 OpenAI Codex 的通用 AI 编码代理，~116 个 Rust crate。正在进行从 "OpenAI Codex" 到 "BCIP 宝宸知识产权" 的品牌迁移，当前**专利能力为零**。

### 1.3 迁移策略

- **模式**: 深度集成 — 将 YunXi 专利引擎重构为 BCIP native crate
- **范围**: 全量一次性迁移 (50+ 工具 / 9 Agent / 全部知识资产)
- **外部依赖**: 纯 Rust 替代 (不用 Python/ONNX/BGE-M3)
- **架构融合**: 作为 BCIP 内建能力，注册到现有 Agent/Tool/Skill 系统
- **排除**: BGE-M3 语义嵌入、向量检索相关功能暂不引入

---

## 2. 新增 Crate 规划

遵循 BCIP 的 codex-rs 开发规范，在 `codex-rs/` 下新增以下 crate：

```
codex-rs/
├── codex-patent-core/          # 专利领域核心类型与 trait
├── codex-patent-knowledge/     # 知识库引擎 (FTS/知识图谱/法规库)
├── codex-patent-domain/        # 专利领域服务 (权利要求/审查/质量/规则引擎)
├── codex-patent-tools/         # 50+ 专利工具注册到 BCIP Tool 系统
├── codex-patent-agents/        # 9 个专利 Agent 角色 + 编排
├── codex-patent-skills/        # 专利 Skill 定义与加载
└── codex-patent-assets/        # 知识资产 (SQLite / 文档 / 卡片)
```

### 设计原则

- 每个 crate 职责单一，模块 <500 行
- 不向 codex-core 添加代码，遵循 AGENTS.md 约束
- 使用 BCIP 现有基础设施 (model-provider、tool handler、agent registry)
- 不依赖 BGE-M3、ONNX、Python 桥接

---

## 3. 融入 BCIP 现有体系

### 3.1 Agent 系统融合

```
BCIP Agent Registry (core/src/agent/)
  ├── 现有: explorer, awaiter (内置)
  └── 新增 ← codex-patent-agents/
       ├── retriever          检索专家 (多源检索/检索式构建)
       ├── analyzer           分析专家 (特征提取/四层对比)
       ├── writer             撰写专家 (说明书/权利要求/摘要)
       ├── novelty_checker    新颖性评估 (三步法逐特征对比)
       ├── creativity_checker 创造性评估 (问题-解决方案法)
       ├── infringement_checker 侵权分析 (全面覆盖+等同原则)
       ├── invalidity_checker 无效分析 (理由/证据分析)
       ├── reviewer           文件审查 (格式/内容质量)
       └── quality_checker    多维度质量评估
```

每个 Agent:
- 实现 BCIP 的 AgentRole trait
- 角色定义使用 TOML (遵循 BCIP 规范，替代 YunXi 的 XML)
- 注册到 BCIP Agent Registry
- 通过标准 model-provider 接口调用 LLM

### 3.2 Tool 系统融合

```
BCIP Tool Registry (core/src/tools/)
  ├── 现有: 50+ 通用工具
  └── 新增 ← codex-patent-tools/
       ├── 实现 BCIP tool handler trait
       ├── 注册到 ToolSpec 体系
       └── 按类别分组注册
```

所有专利工具实现 BCIP 的 tool handler trait，通过标准 ToolSpec 注册，无需修改 core。

### 3.3 Skill 系统融合

```
BCIP Skills (.codex/skills/ 或 codex-skills/)
  ├── 现有: code-review, skill-creator, plugin-creator, ...
  └── 新增 ← codex-patent-skills/
       ├── _shared/           公共模块
       │   ├── legal_reasoning    法律推理模块
       │   ├── hitl_protocol      人机协作协议
       │   ├── output_standards   输出格式标准
       │   ├── quality_checklist  质量检查清单
       │   └── patent_glossary    专利术语表
       ├── cap-retrieval/     检索能力
       ├── cap-analysis/      分析能力
       ├── cap-writing/       撰写能力
       ├── cap-disclosure-exam/ 公开审查
       ├── cap-inventive/     创造性判断
       ├── cap-clarity-exam/  清楚性审查
       ├── cap-invalid/       无效分析
       ├── cap-prior-art-ident/ 现有技术识别
       ├── cap-response/      答复策略
       ├── cap-formal-exam/   形式审查
       └── foundation-hitl/   人机协作基础规则
```

---

## 4. 外部依赖替换方案

### 4.1 语义嵌入 → 不含向量检索

决策: **BGE-M3 语义嵌入和向量检索暂不引入。**

替代方案:
- 全文搜索 (FTS5) 作为主要文本检索方式
- 知识图谱结构化查询 (SQLite Cypher 风格)
- 关键词匹配 + 同义词扩展
- 知识卡片索引 (card-index.json) 用于概念检索
- 默认使用混合搜索中的关键词权重 100%

BCIP 的 model-provider 体系仅在工具需要 LLM 调用时使用，不用于嵌入向量生成。

### 4.2 CNIPA 检索 → BCIP 网络能力

```
YunXi: Python Playwright WAF 绕过 + Rust 桥接
  ↓ 替换为
BCIP: 纯 Rust 方案
  ├── reqwest HTTP 客户端
  ├── Cookie 持久化
  ├── 利用 epub.cnipa.gov.cn 公开接口
  └── 复用 YunXi 已有的 cnipa-query skill 逻辑
```

### 4.3 LLM 调用 → BCIP 统一模型层

```
YunXi: 独立 DeepSeek/Qwen/OpenAI 客户端
  ↓ 替换为
BCIP: codex-model-provider (ChatGPT/Ollama/LM Studio)
  ├── 专利 Agent 通过标准 provider trait 调用 LLM
  └── 支持模型路由 (复杂任务→pro，简单任务→flash)
```

### 4.4 资产迁移

```
YunXi assets/                         →   BCIP codex-patent-assets/
  ├── knowledge-base/ (150+ md)       →   documents/
  ├── knowledge-base/patent_kg.db     →   patent_kg.db
  ├── knowledge/data/laws.db          →   laws.db
  ├── knowledge/data/laws-full.db     →   laws-full.db
  ├── knowledge-base/cards/           →   cards/
  ├── knowledge-base/card-index.json  →   card-index.json
  ├── knowledge-base/Concept-Hierarchy.md → concept-hierarchy.md
  └── agents/*.xml                    →   转为 TOML (BCIP 规范)
```

不迁移的资产:
- `.yunpat-semantic-index.sqlite` (BGE-M3 向量索引，暂不引入)
- Python 脚本和 CNIPA Playwright 客户端

---

## 5. 工具分层迁移清单

按依赖关系分 6 层，从底向上推进：

### 第1层: 底座 — 知识库引擎 (无 LLM 依赖)

| 工具 | 描述 | 数据源 |
|------|------|--------|
| KnowledgeSearch | 统一知识搜索 (FTS/图谱/卡片) | SQLite + FTS5 |
| KnowledgeGraphQuery | 知识图谱查询 | patent_kg.db |
| LawDatabaseQuery | 法律法规查询 | laws.db (FTS5) |
| KnowledgeCardSearch | 知识卡片搜索 | card-index.json |
| SynonymSearch | 同义词词典 (70+术语) | 内嵌词典 |
| SearchQueryBuilder | 3阶段检索式构建 | 规则引擎 |

### 第2层: 检索 — 专利获取

| 工具 | 描述 | 数据源 |
|------|------|--------|
| PatentSearch | 统一专利检索 | 多源聚合 |
| GooglePatentsFetch | Google 专利检索 | Google Patents |
| CnipaSearch | 国知局检索 | epub.cnipa.gov.cn |
| IterativeSearch | 迭代深度检索 | 多轮扩展 |
| HighCitationPatents | 高被引专利 | Google Patents |
| BatchPatentDownload | 批量下载 | Google Patents |

### 第3层: 解析 — 结构化能力

| 工具 | 描述 | 依赖 |
|------|------|------|
| ClaimParse | 权利要求解析 | patent-domain |
| ClaimCompare | 权利要求对比 | ClaimParse |
| ClaimGenerator | 权利要求书生成 | ClaimParse + LLM |
| OaParse | 审查意见解析 | patent-domain |
| DocumentParse | 专利文档解析 | - |
| DrawingUnderstanding | 附图理解 | LLM (视觉) |

### 第4层: 分析 — 专利判断

| 工具 | 描述 | 引擎 |
|------|------|------|
| NoveltyAnalysis | 新颖性分析 | 规则引擎 (三步法) |
| InventivenessAnalysis | 创造性分析 | 规则引擎 (问题-解决方案法) |
| SemanticCompare | 文本对比 (词法/结构/混合) | 规则引擎 |
| PatentCompare | 专利对比矩阵 | ClaimCompare + IPC |
| InfringementAnalysis | 侵权分析 | 规则引擎 (全面覆盖+等同原则) |
| SynergyAnalysis | 技术协同分析 | 规则引擎 |
| LegalQA | 知产法律问答 | 知识库 + LLM |
| LegalReasoning | 结构化法律推理 | 规则引擎 |

### 第5层: 审查与撰写

| 工具 | 描述 | 引擎 |
|------|------|------|
| QualityScorer | 四维质量评分 (12规则) | 规则引擎 |
| FormalCheck | 形式审查 | 规则引擎 |
| SubjectMatterCheck | 保护客体检查 | 规则引擎 |
| UnityCheck | 单一性检查 | 规则引擎 (Jaccard) |
| SpecificationDrafter | 说明书起草 | LLM |
| AbstractDrafter | 摘要起草 | LLM |
| InnovationEvaluator | 创新度评估 | 规则引擎 + LLM |
| OaStrategy | 答复策略 | 规则引擎 + LLM |
| ResponseTemplate | 答复模板 (6内置) | 规则引擎 |
| SuccessPredictor | 成功率预测 | 规则引擎 |

### 第6层: 管理与交付

| 工具 | 描述 | 引擎 |
|------|------|------|
| PatentManager | 生命周期管理 (状态机) | CRUD + 规则 |
| TemplateLibrary | 文档模板 (5内置) | 模板引擎 |
| TrademarkAnalysis | 商标分析 | 规则引擎 |
| ProcessChart | 流程图生成 | 渲染引擎 |
| PatentDownload | 单件下载 | Google Patents |
| ActionReview | 行动审查 | LLM |
| LLMReflection | LLM 反思 | LLM |
| FaithfulnessEval | 忠实度评估 | LLM |
| SelfConsistencyEval | 自一致性评估 | LLM |
| GEval | G-Eval 评估 | LLM |

---

## 6. 知识资产清单

### 6.1 需迁移的资产

| 资产 | 大小 | 格式 | 目标位置 |
|------|------|------|----------|
| patent_kg.db | ~40K 节点 | SQLite | codex-patent-assets/patent_kg.db |
| laws.db | 法规全文 | SQLite (FTS5) | codex-patent-assets/laws.db |
| laws-full.db | 完整法规 | SQLite (FTS5) | codex-patent-assets/laws-full.db |
| card-index.json | 知识卡片索引 | JSON | codex-patent-assets/card-index.json |
| cards/ | 知识卡片 | 文本 | codex-patent-assets/cards/ |
| Concept-Hierarchy.md | 概念层级 | Markdown | codex-patent-assets/concept-hierarchy.md |
| 150+ 知识文档 | 专利知识 | Markdown | codex-patent-assets/documents/ |
| 书籍骨架 (10+本) | 专利经典 | Markdown | codex-patent-assets/documents/books/ |
| agents/*.xml | Agent 定义 | XML | 转为 TOML 到 codex-patent-agents/ |

### 6.2 不迁移的资产

| 资产 | 原因 |
|------|------|
| .yunpat-semantic-index.sqlite | BGE-M3 向量索引，暂不引入 |
| Python/ 目录 | 纯 Rust 策略，Python 代码不迁移 |
| CNIPA Python 客户端 | 用纯 Rust 重写 |

---

## 7. 迁移路线图

```
阶段0: 地基 — 类型与基础设施
  ├── 创建 codex-patent-core crate
  ├── 迁移知识资产文件
  └── 注册 patent 配置项

阶段1: 知识底座
  ├── codex-patent-knowledge crate
  └── 第1层工具 (6个)

阶段2: 检索系统
  └── 第2层工具 (6个)

阶段3: 解析与对比
  ├── codex-patent-domain crate
  └── 第3层工具 (6个)

阶段4: 核心分析引擎
  └── 第4层工具 (8个)

阶段5: 审查与撰写
  └── 第5层工具 (10个)

阶段6: Agent 与 Skill 融合
  ├── codex-patent-agents crate
  └── codex-patent-skills crate

阶段7: 管理与交付
  └── 第6层工具 (10个)

阶段8: 集成测试与验证
  ├── 端到端测试
  ├── 快照测试 (insta)
  └── 专利工作流完整演练
```

---

## 8. 关键约束与规范

### 8.1 codex-rs 开发规范

- Crate 命名: `codex-patent-*`
- 不向 codex-core 添加代码
- 模块 <500 行 (不含测试)
- format! 内联变量: `format!("{var}")`
- match 语句优先使用穷举匹配
- 使用 method references 替代闭包: `.map(SomeStruct::method)`
- 避免 `#[async_trait]`, 使用原生 RPITIT
- 不使用 bool 或模糊 Option 参数 (优先 enum/newtype)
- 预构建语义索引功能暂不引入

### 8.2 测试规范

- 使用 `pretty_assertions::assert_eq` 进行深度比较
- TUI 相关变更需要 insta 快照测试
- 运行 `just fmt` 和 `just fix -p <crate>` 后提交
- 测试通过 `just test -p <crate>` 运行

### 8.3 安全规范

- 不在代码中暴露或记录密钥
- 不引入 `CODEX_SANDBOX_NETWORK_DISABLED_ENV_VAR` 相关逻辑
- 遵循 BCIP 现有的 sandboxing 和执行策略

---

## 9. 已明确决策

1. **FTS vs 语义搜索**: 暂不引入向量检索，FTS5 + 图谱 + 关键词作为检索主力。后续是否引入嵌入模型，依据测试结果再决定。
2. **CNIPA 检索**: 复用已有 cnipa-query skill，不重写。
3. **Agent 角色存储**: 运行时从文件加载 TOML 定义。
4. **测试策略**: 关键路径覆盖优先，逐步扩展到全量 regression 覆盖。
