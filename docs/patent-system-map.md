# BCIP 专利系统四维映射

> Agent ↔ Skill ↔ Tool ↔ Concept ↔ ConstitutionalRule
> 版本: 1.1 | 生成: 2026-05-29 | 更新: 2026-05-30

---

## 映射总览

```
                     ┌──────────────────┐
                     │  Constitutional   │
                     │     Rules         │
                     │  (35条, 13阶段)    │
                     │  + 6 外部化规则    │
                     └───────┬──────────┘
                             │ 约束
         ┌───────────────────┼──────────────────┐
         │                   │                  │
         ▼                   ▼                  ▼
   ┌──────────┐       ┌──────────┐       ┌──────────┐
   │  Agent   │ ←激活→ │  Skill   │ ←需要→ │   Tool   │
   │ (9 roles)│        │(12 skills)│       │(50+ tools)│
   └──────────┘       └──────────┘       └──────────┘
         │                   │                  │
         └───────────────────┼──────────────────┘
                             │ 关联
                             ▼
                    ┌──────────────────┐
                    │   LLM Wiki KB    │
                    │ (100 concepts)   │
                    └──────────────────┘
                             │
                             ▼
                    ┌──────────────────┐
                    │  Text Processing │
                    │ CJK分词+相似度    │
                    │ IPC分类器        │
                    └──────────────────┘
```

---

## 一、Agent → Tool 映射

| Agent | TOML 行数 | 主要工具 | 辅助工具 | 涉及的 Skill |
|-------|----------|---------|---------|-------------|
| **retriever** | 87 | PatentSearch, GooglePatentsFetch, IterativeSearch | SearchQueryBuilder, KnowledgeSearch | cap-retrieval, cap-prior-art-ident |
| **analyzer** | 99 | ClaimParse, ClaimCompare, FeatureExtractor | TechUnit, TechTripleExtractor | cap-analysis |
| **writer** | 112 | ClaimGenerator, SpecificationDrafter, AbstractDrafter | InnovationEvaluator | cap-writing, stop-slop |
| **novelty_checker** | 107 | NoveltyAnalysis, ClaimCompare | KnowledgeSearch | cap-analysis, cap-prior-art-ident |
| **creativity_checker** | 133 | InventivenessAnalysis, ClaimCompare | KnowledgeSearch | cap-inventive |
| **infringement_checker** | 106 | InfringementAnalysis, ClaimParse | KnowledgeSearch | (隐含侵权分析能力) |
| **invalidity_checker** | 110 | ClaimCompare, KnowledgeSearch | LegalKnowledgeSearch | cap-invalid |
| **reviewer** | 99 | FormalCheck, ClaimParse | QualityAssess | cap-clarity-exam, cap-disclosure-exam, cap-formal-exam |
| **quality_checker** | 84 | QualityAssess, FormalCheck | KnowledgeSearch | cap-clarity-exam |

---

## 二、Tool → ConstitutionalRule 映射

### 撰写域

| Tool | 关联规则 | 阶段 |
|------|---------|------|
| ClaimGenerator | CON-203(清楚简要) CON-204(以说明书为依据) CON-205(必要特征) CON-206(引用规则) CON-1301(禁用表述) | 撰写 |
| SpecificationDrafter | CON-201(充分公开) CON-202(格式要求) CON-1302(禁用表述) | 撰写 |
| AbstractDrafter | CON-202(格式要求) | 撰写 |

### 审查域

| Tool | 关联规则 | 阶段 |
|------|---------|------|
| NoveltyAnalysis | CON-301(新颖性) CON-302(宽限期) | 审查 |
| InventivenessAnalysis | CON-401(创造性) CON-402(实用新型) | 审查 |
| InnovationEvaluator | CON-401(创造性) | 申请前 |
| SubjectMatterChecker | CON-101(三要素) CON-102(违法) CON-103(25条) CON-104(计算机程序) | 申请前 |
| UnityChecker | CON-601(单一性) | 审查 |

### 答复域

| Tool | 关联规则 | 阶段 |
|------|---------|------|
| OaParser | CON-901(答复期限) | 答复 |
| OaStrategist | CON-902(新颖性答复) CON-903(创造性答复) CON-904(清楚答复) | 答复 |
| PatentResponder | CON-701(不超范围) CON-702(范围缩小) CON-901~904 | 答复 |

### 无效域

| Tool | 关联规则 | 阶段 |
|------|---------|------|
| KnowledgeSearch (无效) | CON-1101(无效理由) CON-1102(无效修改) | 无效 |

### 侵权域

| Tool | 关联规则 | 阶段 |
|------|---------|------|
| PatentInfringement | CON-1201(侵权判定) CON-1202(损害赔偿) | 维权 |
| InfringementAnalysis | CON-1201(侵权判定) | 维权 |

---

## 三、Skill → Concept 映射

| Skill | 关联概念 (from Concept-Index) | Concept-Hierarchy 域 |
|-------|---------------------------|-------------------|
| cap-retrieval | 现有技术, 出版物公开, 抵触申请, IPC分类 | 现有技术 |
| cap-analysis | 权利要求解释, 技术特征, 功能性特征, 特征部分 | 权利要求与说明书 |
| cap-writing | 充分公开, 独立权利要求, 从属权利要求, 必要技术特征 | 权利要求与说明书 |
| cap-inventive | 创造性, 三步法, 技术启示, 非显而易见, 公知常识 | 专利授权 |
| cap-prior-art-ident | 现有技术, 抵触申请, 新颖性 | 现有技术 |
| cap-clarity-exam | 清楚, 权利要求解释, 功能性特征 | 权利要求与说明书 |
| cap-disclosure-exam | 充分公开, 能够实现 | 专利授权 |
| cap-formal-exam | 说明书, 权利要求, 附图 | 权利要求与说明书 |
| cap-invalid | 专利无效, 无效宣告理由, 证据认定 | 专利无效 |
| cap-response | 审查意见, 修改, 答复策略 | 专利审查 |

---

## 四、Phase → 适用工具矩阵

| 阶段 | 激活的 Rule | 激活的 Tool | 激活的 Agent | 激活的 Skill |
|------|-----------|-----------|-------------|-------------|
| **申请前** | CON-101~104 IPCC | SubjectMatterChecker, InnovationEvaluator | — | cap-prior-art-ident |
| **撰写** | CON-201~206, 1301~1302 | ClaimGenerator, SpecificationDrafter, AbstractDrafter | writer | cap-writing, stop-slop |
| **审查** | CON-301~302, 401~402, 501, 601~602 | NoveltyAnalysis, InventivenessAnalysis, UnityChecker | novelty_checker, creativity_checker | cap-analysis, cap-inventive |
| **答复** | CON-701~703, 901~904 | OaParser, OaStrategist, PatentResponder | — | cap-response |
| **无效** | CON-1001, 1101~1102 | KnowledgeSearch(Legal) | invalidity_checker | cap-invalid |
| **维权** | CON-1201~1202 | PatentInfringement, InfringementAnalysis | infringement_checker | — |

---

## 五、Crates 依赖关系

```
codex-patent-core (基础类型)
    ├── codex-patent-domain (领域逻辑)
    ├── codex-patent-text (文本处理)
    ├── codex-patent-knowledge (知识层)
    ├── codex-patent-constitutional (宪法规则)
    └── codex-patent-tools (工具函数)
        └── codex-patent-agents (Agent 角色)
        └── codex-patent-skills (技能定义)
        └── codex-patent-scheduler (任务调度)
codex-patent-assets (静态资产, 被 constitutional/knowledge 引用)
```
