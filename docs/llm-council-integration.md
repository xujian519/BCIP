# LLM Council 模式集成方案

> 基于 Andrej Karpathy 的 [LLM Council](https://github.com/karpathy/llm-council) 项目分析，对 BCIP 专利智能体系统的能力提升方案。
>
> 分析日期：2026-05-29 | 来源：`/Users/xujian/Downloads/llm-council-master` | 状态：待实施 | 版本：0.1

---

## 1. 来源项目概述

LLM Council 是 Karpathy 的周末项目，本质是一个**三阶段 LLM 审议系统**：

```
用户提问 → Stage 1 (初评: 多模型并行独立回答)
        → Stage 2 (互评: 匿名化后各模型互审排名)
        → Stage 3 (终裁: Chairman 综合输出最终答案)
```

### 1.1 核心技术特征

| 特征 | 实现方式 | 价值 |
|------|----------|------|
| 多模型并行 | `asyncio.gather` + OpenRouter | 延迟最小化 |
| 匿名化互评 | 模型名 → Response A/B/C 映射 | 消除模型偏好偏见 |
| Aggregate Ranking | 多模型排名取平均位置 | 量化共识度 |
| 优雅降级 | 单模型失败不阻断整体 | 系统鲁棒性 |
| SSE 流式 | Server-Sent Events 实时推送阶段进度 | 用户体验 |
| 全透明 UI | Tab 视图展示每阶段原始输出 | 可追溯性 |

### 1.2 项目结构

```
backend/
├── config.py          # COUNCIL_MODELS, CHAIRMAN_MODEL
├── openrouter.py      # OpenRouter 客户端 (单模型/并行)
├── council.py         # ★ 核心三阶段编排
├── storage.py         # JSON 文件存储
└── main.py            # FastAPI + SSE 流式端点

frontend/src/components/
├── Stage1.jsx         # 初评 Tab 视图
├── Stage2.jsx         # 互评 + 聚合排名
└── Stage3.jsx         # Chairman 终裁
```

---

## 2. BCIP 现状与差距

### 2.1 现有技能痛点

| 技能 | 当前模式 | 核心差距 |
|------|----------|----------|
| `storm-patent-experts` | 单模型 prompt 切换多角色 | 缺乏真实认知多样性 |
| `patent-drafting-v2` | 单模型顺序撰写 | 无独立质量评审 |
| `patent-analysis` | 单模型分析 | 结论缺乏交叉印证 |
| `patent-response` | 单模型策略 | 策略无竞争性评估 |
| `patent-invalid` | 单模型检索分析 | 无效力度评估单一 |
| `patent-plan-mode` | 5 阶段工作流 | 阶段间无自动质量门控 |
| `grill-me` | 单模型多角色质询 | 角色多样性受限 |

### 2.2 核心差距

1. **认知单一性**：所有"多角色"是同一底座模型通过 prompt 切换
2. **缺乏交叉验证**：专利分析只有单一产出，无独立多方评审
3. **无量化共识**：无法告诉用户"几个模型同意这个结论"
4. **质量门控缺失**：关键决策点只有 HITL，无自动化 AI 评审

---

## 3. 集成方案

### 3.1 总体架构

创建 **`patent-council`** 技能作为基础设施，各专利技能按需接入：

```
                 ┌──────────────────────────┐
                 │    patent-council skill   │
                 │  ┌──────────────────────┐ │
                 │  │ 多模型并行编排        │ │
                 │  │ 匿名化互评引擎        │ │
                 │  │ Aggregate Ranking    │ │
                 │  │ Chairman 终裁         │ │
                 │  │ 优雅降级              │ │
                 │  └──────────────────────┘ │
                 └──────────┬───────────────┘
                            │
     ┌──────────────────────┼──────────────────────┐
     ▼                      ▼                      ▼
 patent-drafting      patent-analysis       patent-response
 (撰写质量门控)        (多模型交叉分析)       (策略竞争评估)
     │                      │                      │
     ▼                      ▼                      ▼
 patent-invalid       storm-experts         patent-plan-mode
 (无效力度评估)        (真实多专家)           (流程质量节点)
```

### 3.2 概念 API

```python
class PatentCouncil:
    async def deliberate(self, task: PatentTask) -> CouncilResult:
        """三阶段审议：初评 → 匿名互评 → Chairman 终裁"""

    async def quality_gate(
        self, draft: PatentDocument, threshold: float = 0.7
    ) -> GateResult:
        """质量门控：多模型独立评审，低于阈值退回 + 修改建议"""

    async def ranking(
        self, items: List[Any], criteria: List[str]
    ) -> RankedResult:
        """多模型排序（Borda Count + 平均排名双输出）"""
```

### 3.3 分步实施计划

| Phase | 目标 | 优先级 | 涉及技能 |
|-------|------|--------|----------|
| 1 | 创建 `patent-council` 通用框架 | **最高** | 新建 |
| 2 | 改造 `storm-patent-experts` 为真实多模型 | **高** | storm-patent-experts |
| 3 | 专利撰写质量门控 | **高** | patent-drafting-v2, patent-drafting-workflow |
| 4 | 审查意见答复策略竞争 | 中 | patent-response |
| 5 | 无效宣告多模型增强 | 中 | patent-invalid |
| 6 | patent-plan-mode 集成 | 低 | patent-plan-mode |

#### Phase 2 详情：storm-patent-experts 改造

```
原流程: 单模型 → 角色A → 角色B → 角色C → 角色D → 综合

新流程:
  模型A(审查员) ─┐
  模型B(技术专家) ─┼─→ 匿名互评 ─→ Chairman(代理师) ─→ 终裁 + 共识度
  模型C(专利律师) ─┘
```

#### Phase 3 详情：撰写质量门控

```
权利要求撰写 → [Council Gate: 清楚性/简要性/创造性, 阈值 ≥ 75%]
                    │
              < 75% → 标记修改建议 → 退回重写
              ≥ 75% → 通过 → 说明书撰写 → [Council Gate: 公开充分, 阈值 ≥ 70%]
```

#### Phase 4 详情：答复策略竞争

```
审查意见 → 模型A 策略1 ─┐
           模型B 策略2 ─┼─→ 互评(审查员视角质疑) ─→ Chairman 选优整合
           模型C 策略3 ─┘
```

---

## 4. 预期能力提升

| 维度 | 当前 | 集成后 | 提升 |
|------|------|--------|------|
| 分析准确度 | 单模型，无交叉验证 | 多模型交叉 + 共识度量化 | ★★★★☆ |
| 撰写质量 | 单模型 + 人工检查 | 自动化多模型质量门控 | ★★★★★ |
| 审查意见质量 | 单模型策略 | 多策略竞争 + 审查员视角互评 | ★★★★☆ |
| 角色多样性 | 单模型 prompt 切换 | 真实多模型认知差异 | ★★★★★ |
| 决策可解释性 | 仅有结论 | 结论 + 评分 + 共识度 | ★★★★☆ |
| 系统鲁棒性 | 单点故障 | 优雅降级 | ★★★☆☆ |
| 用户体验 | 等待全程 | SSE 流式进度推送 | ★★★☆☆ |

---

## 5. 风险与缓解

| 风险 | 缓解 |
|------|------|
| 成本 N 倍增加 | 简单任务单模型；关键节点启用 Council；便宜模型初评 |
| 延迟增加 | Stage 1 完全并行，延迟 ≈ 最慢模型；可设超时 |
| 匿名化被绕过 | 实践中影响有限（Karpathy 已验证） |
| 排名一致性差 | 分歧本身是重要信号，Chairman 阶段价值更大 |
| Chairman 偏见 | 轮值 Chairman 或用户自选 |

---

## 6. 附录：LLM Council 核心代码模式

### 三阶段编排

```python
async def run_full_council(user_query):
    stage1 = await stage1_collect_responses(user_query)       # 并行初评
    stage2, mapping = await stage2_collect_rankings(...)      # 匿名互评
    aggregate = calculate_aggregate_rankings(stage2, mapping) # 聚合排名
    stage3 = await stage3_synthesize_final(...)               # Chairman 终裁
    return stage1, stage2, stage3, metadata
```

### 匿名化互评 Prompt

```
You are evaluating different responses to: {question}

Response A: {匿名化的模型A输出}
Response B: {匿名化的模型B输出}
Response C: {匿名化的模型C输出}

1. Evaluate each response individually
2. Provide: FINAL RANKING:
   1. Response C
   2. Response A
   3. Response B
```

### 并行调用

```python
async def query_models_parallel(models, messages):
    tasks = [query_model(m, messages) for m in models]
    responses = await asyncio.gather(*tasks)
    return dict(zip(models, responses))
```

### SSE 流式进度

```python
async def event_generator():
    yield sse("stage1_start")
    stage1 = await stage1_collect_responses(content)
    yield sse("stage1_complete", data=stage1)
    # ... stage2, stage3 ...
    yield sse("complete")
```
