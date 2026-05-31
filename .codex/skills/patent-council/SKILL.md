---
name: patent-council
description: 专利多模型审议框架 — 三阶段 LLM Council 模式：多模型并行独立分析→匿名化互评排名→Chairman终裁综合输出。用于需要多视角交叉验证的专利任务，包括创造性判断、审查意见答复策略、撰写质量门控、无效分析、专利分析等。触发词："Council审议"、"多模型评审"、"交叉验证"、"质量门控"、"patent-council"、"模型投票"
---

# Patent Council — 专利多模型审议引擎

基于 LLM Council 三阶段审议模式的专利领域实现。

## 核心流程

```
用户任务
  │
  ├─ Stage 1: 初评 ── 所有Council模型并行独立分析
  ├─ Stage 2: 互评 ── 匿名化后各模型互审排名
  └─ Stage 3: 终裁 ── Chairman综合所有信息输出权威结论
```

## 何时使用

此技能用于需要**多模型交叉验证**的专利场景。单模型分析即可满足的简单任务无需使用。

**必须使用的场景**：
- 用户明确要求 "Council审议"、"多模型评审"、"交叉验证"
- 专利创造性判断（主观性强，单模型易偏差）
- 审查意见答复策略（策略质量需要竞争性评估）
- 无效宣告分析（多角度检索和论证）

**推荐使用的场景**：
- 专利撰写后的质量门控（权利要求书、说明书）
- 复杂技术方案的专利分析
- 对比文件的相似度判断

## 使用方式

### 方式1：完整三阶段审议

```bash
python scripts/council.py -t "判断权利要求1相对于D1的创造性" \
  -m "gpt-4o,claude-sonnet-4.5,gemini-2.5-pro" \
  -c "claude-sonnet-4.5" \
  --verbose
```

### 方式2：作为库导入

```python
from council import PatentCouncil, CouncilConfig, run_council
import asyncio

config = CouncilConfig(
    models=["gpt-4o", "claude-sonnet-4.5", "gemini-2.5-pro"],
    chairman="claude-sonnet-4.5",
    criteria=["准确性", "法律依据", "论证深度"],
)

result = asyncio.run(run_council(config, "判断权利要求的创造性..."))
print(result.summary())
```

### 方式3：质量门控

```python
from council import quality_gate

result = await quality_gate(config, document_text, "权利要求书", threshold=0.7)
# result: {"passed": bool, "score": float, "issues": [...]}
```

## 模型配置指南

| 任务 | 推荐 Council | 推荐 Chairman |
|------|-------------|---------------|
| 创造性判断 | gpt-4o, claude, gemini | claude |
| 审查意见答复 | gpt-4o, gpt-5.1, claude | gpt-5.1 |
| 撰写质量门控 | claude, gemini, gpt-4o | claude |
| 无效分析 | claude, gemini, gpt-4o | gemini |
| 通用分析 | gpt-4o, claude, gemini | claude |

**成本控制**：
- 简单任务：用 2 个便宜模型初评 + 1 个强模型做 Chairman
- 关键任务：用 3-4 个强模型全流程

## 输出解读

### 聚合排名
排名数字越小越好。Borda Score 是标准化评分（越高越好）。

### 共识度
- **HIGH (≥0.8)**：模型高度一致，结论可信
- **MODERATE (0.5-0.8)**：有部分分歧，参考 Chairman 判断
- **LOW (<0.5)**：分歧较大，建议人工介入

### 完整结果
`--output results.json` 保存完整 JSON 结果，包含每个阶段的原始输出。

## 关键设计

### 匿名化互评 (Stage 2)
模型A的输出标注为 "Response A"，评审者不知道在评价谁，消除偏好偏见。

### 优雅降级
单个模型失败不影响整体。至少 `min_successful` (默认2) 个模型成功才继续。

### Chairman 轮值
避免单一 Chairman 偏见。可根据任务类型更换 Chairman，或比较不同 Chairman 的输出。

## 与 BCIP 技能的集成

### 集成到 patent-drafting-v2
在权利要求撰写和说明书撰写后插入质量门控：
```python
gate = await quality_gate(config, claims_text, "权利要求书", 0.75)
if not gate["passed"]:
    # 返回修改建议给撰写流程
    ...
```

### 集成到 patent-response
多模型生成竞争策略，互评选优：
```python
task = "针对以下审查意见生成答复策略：[OA内容]"
result = await run_council(config, task)
# result.stage3 是最优策略
```

### 集成到 storm-patent-experts
将单模型多角色升级为真实多模型：
```python
config = CouncilConfig(
    models=["gpt-4o", "claude-sonnet-4.5", "gemini-2.5-pro"],
    chairman="claude-sonnet-4.5",
)
result = await run_council(config, "分析专利CNxxx的技术方案")
```

## Prompt 模板

详见 [references/prompt-templates.md](references/prompt-templates.md)，包含：
- 创造性判断专用模板
- 审查意见答复策略模板
- 无效宣告分析模板
- 质量门控评审维度
- Chairman 选择指南

## 依赖

- Python 3.10+
- httpx (异步 HTTP)
- OpenAI-compatible API (OPENAI_API_KEY 环境变量)
- 安装: `pip install httpx`

## 输出格式

- 终端: 人类可读的摘要 + Chairman 终裁
- JSON 文件 (--output): 完整结构化数据
- 包含: 每个模型的响应、延迟、排名、共识度
