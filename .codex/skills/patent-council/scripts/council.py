#!/usr/bin/env python3
"""
Patent Council — 专利多模型审议引擎

实现三阶段 LLM Council 审议模式：
  Stage 1: 多模型并行独立分析
  Stage 2: 匿名化互评与排名
  Stage 3: Chairman 终裁综合输出

用法:
  python council.py --task "判断权利要求1的创造性" --models gpt-5.1,claude-sonnet-4.5,gemini-3-pro \
      --chairman claude-sonnet-4.5 --criteria "准确性,法律依据,论证深度"

或作为库导入:
  from council import PatentCouncil, CouncilConfig
"""

import asyncio
import json
import os
import re
import sys
import time
from collections import defaultdict
from dataclasses import dataclass, field
from typing import Any, Dict, List, Optional

# ── 配置 ──────────────────────────────────────────────


@dataclass
class CouncilConfig:
    """Council 配置"""

    models: List[str] = field(
        default_factory=lambda: [
            "openai/gpt-4o",
            "anthropic/claude-sonnet-4.5",
            "google/gemini-2.5-pro",
        ]
    )
    chairman: str = "anthropic/claude-sonnet-4.5"
    criteria: List[str] = field(
        default_factory=lambda: [
            "准确性",
            "法律依据充分性",
            "论证逻辑严密性",
            "完整性",
        ]
    )
    timeout: float = 120.0
    min_successful: int = 2  # 至少需要几个模型成功
    temperature: float = 0.7
    max_tokens: int = 4096
    api_base: str = "https://api.openai.com/v1"
    api_key: str = ""
    verbose: bool = False

    def __post_init__(self):
        self.api_key = self.api_key or os.getenv("OPENAI_API_KEY", "")
        self.api_base = self.api_base or os.getenv(
            "OPENAI_BASE_URL", "https://api.openai.com/v1"
        )


# ── 数据模型 ──────────────────────────────────────────


@dataclass
class ModelResponse:
    """单个模型的响应"""

    model: str
    content: str
    latency_ms: float
    success: bool
    error: Optional[str] = None


@dataclass
class RankingResult:
    """一个模型对其他模型的排名"""

    reviewer: str  # 评审者模型
    full_text: str  # 完整评审文本
    parsed_ranking: List[str]  # 解析出的排名列表 ["Response B", "Response A", ...]


@dataclass
class AggregateRanking:
    """聚合排名"""

    model: str
    average_rank: float
    borda_score: float
    rankings_count: int


@dataclass
class Stage1Result:
    """Stage 1 结果"""

    responses: List[ModelResponse]
    total_latency_ms: float


@dataclass
class Stage2Result:
    """Stage 2 结果"""

    rankings: List[RankingResult]
    label_mapping: Dict[str, str]  # "Response A" → "openai/gpt-4o"
    aggregate: List[AggregateRanking]
    total_latency_ms: float


@dataclass
class Stage3Result:
    """Stage 3 结果"""

    response: ModelResponse
    total_latency_ms: float


@dataclass
class CouncilResult:
    """完整 Council 审议结果"""

    task: str
    config: CouncilConfig
    stage1: Stage1Result
    stage2: Stage2Result
    stage3: Stage3Result
    metadata: Dict[str, Any] = field(default_factory=dict)

    def to_dict(self) -> dict:
        return {
            "task": self.task,
            "stage1": {
                "responses": [
                    {
                        "model": r.model,
                        "content": r.content,
                        "latency_ms": r.latency_ms,
                        "success": r.success,
                    }
                    for r in self.stage1.responses
                ],
                "total_latency_ms": self.stage1.total_latency_ms,
            },
            "stage2": {
                "rankings": [
                    {
                        "reviewer": r.reviewer,
                        "full_text": r.full_text,
                        "parsed_ranking": r.parsed_ranking,
                    }
                    for r in self.stage2.rankings
                ],
                "label_mapping": self.stage2.label_mapping,
                "aggregate": [
                    {
                        "model": a.model,
                        "average_rank": a.average_rank,
                        "borda_score": a.borda_score,
                        "rankings_count": a.rankings_count,
                    }
                    for a in self.stage2.aggregate
                ],
                "total_latency_ms": self.stage2.total_latency_ms,
            },
            "stage3": {
                "model": self.stage3.response.model,
                "content": self.stage3.response.content,
                "latency_ms": self.stage3.response.latency_ms,
            },
            "consensus": self._compute_consensus(),
        }

    def _compute_consensus(self) -> dict:
        """计算共识度指标"""
        if not self.stage2.aggregate:
            return {"level": "unknown", "score": 0.0}
        ranks = [a.average_rank for a in self.stage2.aggregate]
        if not ranks:
            return {"level": "unknown", "score": 0.0}
        # 排名标准差越小，共识越高
        mean = sum(ranks) / len(ranks)
        variance = sum((r - mean) ** 2 for r in ranks) / len(ranks)
        std_dev = variance**0.5

        n = len(self.stage2.aggregate)
        score = max(0.0, 1.0 - std_dev / n)

        if score >= 0.8:
            level = "high"
        elif score >= 0.5:
            level = "moderate"
        else:
            level = "low"

        return {"level": level, "score": round(score, 2), "std_dev": round(std_dev, 2)}

    def summary(self) -> str:
        """生成人类可读的摘要"""
        consensus = self._compute_consensus()
        lines = [
            "=" * 60,
            "Patent Council 审议结果",
            "=" * 60,
            f"任务: {self.task[:80]}...",
            "",
            f"[Stage 1] {len(self.stage1.responses)} 个模型完成初评 "
            f"({self.stage1.total_latency_ms / 1000:.1f}s)",
            "",
        ]

        if self.stage2.aggregate:
            lines.append("[Stage 2] 聚合排名（数字越小越好）:")
            for a in self.stage2.aggregate:
                lines.append(
                    f"  {a.average_rank:.2f}  {a.model} "
                    f"(Borda: {a.borda_score:.1f}, {a.rankings_count}票)"
                )
            lines.append("")

        lines.append(
            f"[Stage 3] Chairman ({self.stage3.response.model}) 终裁完成 "
            f"({self.stage3.response.latency_ms / 1000:.1f}s)"
        )
        lines.append(
            f"[共识度] {consensus['level'].upper()} (score={consensus['score']})"
        )
        lines.append("")
        lines.append("─" * 40)
        lines.append("Chairman 终裁:")
        lines.append(self.stage3.response.content[:500])
        lines.append("─" * 40)

        return "\n".join(lines)


# ── LLM 调用层 ────────────────────────────────────────


async def _query_model(
    config: CouncilConfig,
    model: str,
    system_prompt: str,
    user_prompt: str,
) -> ModelResponse:
    """查询单个模型"""
    t0 = time.time()
    try:
        import httpx

        headers = {
            "Authorization": f"Bearer {config.api_key}",
            "Content-Type": "application/json",
        }
        payload = {
            "model": model,
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": user_prompt},
            ],
            "temperature": config.temperature,
            "max_tokens": config.max_tokens,
        }

        async with httpx.AsyncClient(timeout=config.timeout) as client:
            resp = await client.post(
                f"{config.api_base}/chat/completions",
                headers=headers,
                json=payload,
            )
            resp.raise_for_status()
            data = resp.json()
            content = data["choices"][0]["message"]["content"]
            latency = (time.time() - t0) * 1000

            return ModelResponse(
                model=model,
                content=content,
                latency_ms=latency,
                success=True,
            )
    except Exception as e:
        latency = (time.time() - t0) * 1000
        if config.verbose:
            print(f"  [WARN] {model} 失败: {e}", file=sys.stderr)
        return ModelResponse(
            model=model,
            content="",
            latency_ms=latency,
            success=False,
            error=str(e),
        )


async def _query_models_parallel(
    config: CouncilConfig,
    models: List[str],
    system_prompt: str,
    user_prompt: str,
) -> List[ModelResponse]:
    """并行查询多个模型"""
    tasks = [_query_model(config, m, system_prompt, user_prompt) for m in models]
    return await asyncio.gather(*tasks)


# ── Stage 1: 初评 ─────────────────────────────────────


STAGE1_SYSTEM = """你是一位专利领域的资深专家。请根据用户提供的任务，进行独立、深入的分析。

要求：
1. 给出明确的结论
2. 提供充分的法律依据和推理过程
3. 指出关键证据和论据
4. 如果有不确定性，明确标注并说明原因

请使用中文输出。"""


async def stage1_collect_responses(config: CouncilConfig, task: str) -> Stage1Result:
    """Stage 1: 并行收集所有模型的独立回答"""
    if config.verbose:
        print(f"[Stage 1] 并行查询 {len(config.models)} 个模型...", file=sys.stderr)

    t0 = time.time()
    responses = await _query_models_parallel(config, config.models, STAGE1_SYSTEM, task)
    latency = (time.time() - t0) * 1000

    successful = [r for r in responses if r.success]
    if config.verbose:
        print(
            f"[Stage 1] 完成: {len(successful)}/{len(responses)} 成功 "
            f"({latency / 1000:.1f}s)",
            file=sys.stderr,
        )

    return Stage1Result(responses=responses, total_latency_ms=latency)


# ── Stage 2: 匿名互评 ─────────────────────────────────


def _generate_labels(n: int) -> List[str]:
    """生成匿名标签 A, B, C, ..."""
    return [chr(65 + i) for i in range(n)]


STAGE2_SYSTEM = """你是一位专利质量评审专家。你会看到同一问题的多个匿名回答，请：

1. 逐一评估每个回答的优缺点，按以下维度打分（1-10分）：
   {criteria_text}
2. 在评估末尾，严格按照以下格式给出最终排名：

FINAL_RANKING:
1. Response X
2. Response Y
...

请使用中文输出。"""


async def stage2_collect_rankings(
    config: CouncilConfig,
    task: str,
    stage1: Stage1Result,
) -> Stage2Result:
    """Stage 2: 每个模型匿名评审并排名其他模型的回答"""
    successful = [r for r in stage1.responses if r.success]
    if len(successful) < config.min_successful:
        raise ValueError(
            f"成功模型数 ({len(successful)}) 少于最低要求 ({config.min_successful})"
        )

    labels = _generate_labels(len(successful))
    label_mapping = {
        f"Response {label}": r.model for label, r in zip(labels, successful)
    }

    # 构建匿名化回答块
    responses_text = "\n\n".join(
        [f"### Response {label}\n{r.content}" for label, r in zip(labels, successful)]
    )

    criteria_text = "\n".join([f"   - {c}" for c in config.criteria])

    system_prompt = STAGE2_SYSTEM.format(criteria_text=criteria_text)
    user_prompt = f"""原始任务：
{task}

以下是各模型的匿名回答：

{responses_text}

请逐一评估每个回答，然后给出最终排名。"""

    if config.verbose:
        print(f"[Stage 2] {len(successful)} 个模型互评中...", file=sys.stderr)

    t0 = time.time()
    review_responses = await _query_models_parallel(
        config,
        [r.model for r in successful],
        system_prompt,
        user_prompt,
    )
    latency = (time.time() - t0) * 1000

    rankings = []
    for resp in review_responses:
        if resp.success:
            parsed = _parse_ranking(resp.content)
            rankings.append(
                RankingResult(
                    reviewer=resp.model,
                    full_text=resp.content,
                    parsed_ranking=parsed,
                )
            )

    # 计算聚合排名
    aggregate = _compute_aggregate_rankings(rankings, label_mapping)

    if config.verbose:
        print(
            f"[Stage 2] 完成: {len(rankings)} 个评审 ({latency / 1000:.1f}s)",
            file=sys.stderr,
        )

    return Stage2Result(
        rankings=rankings,
        label_mapping=label_mapping,
        aggregate=aggregate,
        total_latency_ms=latency,
    )


def _parse_ranking(text: str) -> List[str]:
    """从评审文本中解析排名"""
    # 方法1: 查找 "FINAL RANKING:" 区块
    if re.search(r"FINAL[_s]*RANKING", text, re.IGNORECASE):
        parts = re.split(r"FINAL[_s]*RANKING\s*:", text, flags=re.IGNORECASE)
        if len(parts) >= 2:
            section = parts[1]
            matches = re.findall(r"\d+\.\s*(Response [A-Z])", section)
            if matches:
                return list(matches)
            # fallback: 任何 Response X 按出现顺序
            matches = re.findall(r"Response [A-Z]", section)
            if matches:
                return list(matches)

    # 方法2: 全文查找 Response X
    matches = re.findall(r"Response [A-Z]", text)
    # 去重保持顺序
    seen = set()
    unique = []
    for m in matches:
        if m not in seen:
            seen.add(m)
            unique.append(m)
    return unique


def _compute_aggregate_rankings(
    rankings: List[RankingResult],
    label_mapping: Dict[str, str],
) -> List[AggregateRanking]:
    """计算聚合排名：平均排名 + Borda Count"""
    model_positions: Dict[str, List[int]] = defaultdict(list)

    for ranking in rankings:
        for pos, label in enumerate(ranking.parsed_ranking, start=1):
            model_name = label_mapping.get(label)
            if model_name:
                model_positions[model_name].append(pos)

    n = len(label_mapping)
    result = []
    for model_name, positions in model_positions.items():
        if not positions:
            continue
        avg_rank = sum(positions) / len(positions)
        # Borda: 排名第k得 n-k 分
        borda = sum(n - p for p in positions) / len(positions)
        result.append(
            AggregateRanking(
                model=model_name,
                average_rank=round(avg_rank, 2),
                borda_score=round(borda, 2),
                rankings_count=len(positions),
            )
        )

    result.sort(key=lambda x: x.average_rank)
    return result


# ── Stage 3: Chairman 终裁 ─────────────────────────────


STAGE3_SYSTEM = """你是 Patent Council 的首席审议官 (Chairman)。你将收到：
1. 各模型的独立分析（Stage 1）
2. 各模型对其他模型的匿名评审和排名（Stage 2）

你的任务是综合所有信息，输出一份权威的最终审议报告。

报告结构：
## 审议结论
[明确的结论]

## 主要依据
[支持结论的核心论据，注明来自哪些模型]

## 分歧分析
[模型间存在分歧的地方，以及你的判断]

## 风险提示
[不确定性、需要人工确认的点]

## 共识度
[模型间的一致性评估]

请使用中文输出，引用具体模型观点时标注来源。"""


async def stage3_synthesize_final(
    config: CouncilConfig,
    task: str,
    stage1: Stage1Result,
    stage2: Stage2Result,
) -> Stage3Result:
    """Stage 3: Chairman 综合所有信息输出终裁"""
    successful = [r for r in stage1.responses if r.success]

    # 构建 Stage 1 汇总
    s1_text = "\n\n".join([f"### {r.model}\n{r.content}" for r in successful])

    # 构建 Stage 2 汇总
    s2_lines = []
    for r in stage2.rankings:
        s2_lines.append(f"### {r.reviewer} 的评审\n{r.full_text}")
    s2_text = "\n\n".join(s2_lines)

    # 构建聚合排名
    agg_lines = ["聚合排名（数字越小越好）："]
    for a in stage2.aggregate:
        agg_lines.append(
            f"  {a.average_rank:.2f}  {a.model} "
            f"(Borda: {a.borda_score:.1f}, {a.rankings_count}票)"
        )
    agg_text = "\n".join(agg_lines)

    user_prompt = f"""原始任务：
{task}

═══════════════════════════════════
STAGE 1 — 各模型独立分析
═══════════════════════════════════
{s1_text}

═══════════════════════════════════
STAGE 2 — 各模型匿名互评
═══════════════════════════════════
{s2_text}

═══════════════════════════════════
聚合排名
═══════════════════════════════════
{agg_text}

请综合以上信息，输出最终审议报告。"""

    if config.verbose:
        print(f"[Stage 3] Chairman ({config.chairman}) 终裁中...", file=sys.stderr)

    t0 = time.time()
    response = await _query_model(config, config.chairman, STAGE3_SYSTEM, user_prompt)
    latency = (time.time() - t0) * 1000

    if not response.success:
        # Chairman 失败，尝试用排名第一的模型作为备选
        if stage2.aggregate:
            fallback = stage2.aggregate[0].model
            if config.verbose:
                print(
                    f"[Stage 3] Chairman 失败，使用排名第一模型 {fallback} 替代",
                    file=sys.stderr,
                )
            response = await _query_model(config, fallback, STAGE3_SYSTEM, user_prompt)
            response.model = f"{fallback} (fallback chairman)"

    if config.verbose:
        print(f"[Stage 3] 完成 ({latency / 1000:.1f}s)", file=sys.stderr)

    return Stage3Result(response=response, total_latency_ms=latency)


# ── 顶层 API ──────────────────────────────────────────


async def run_council(config: CouncilConfig, task: str) -> CouncilResult:
    """
    运行完整的 Patent Council 三阶段审议。

    Args:
        config: Council 配置
        task: 专利分析任务描述

    Returns:
        CouncilResult 包含三个阶段的完整结果
    """
    # Stage 1
    stage1 = await stage1_collect_responses(config, task)

    successful = [r for r in stage1.responses if r.success]
    if len(successful) < config.min_successful:
        raise RuntimeError(
            f"成功模型数 ({len(successful)}) 不足，需要至少 {config.min_successful}"
        )

    # Stage 2
    stage2 = await stage2_collect_rankings(config, task, stage1)

    # Stage 3
    stage3 = await stage3_synthesize_final(config, task, stage1, stage2)

    return CouncilResult(
        task=task,
        config=config,
        stage1=stage1,
        stage2=stage2,
        stage3=stage3,
    )


# ── 便捷函数 ──────────────────────────────────────────


async def quality_gate(
    config: CouncilConfig,
    document: str,
    document_type: str = "权利要求书",
    threshold: float = 0.7,
) -> Dict[str, Any]:
    """
    专利文档质量门控。

    多模型独立评审文档质量，计算通过率，低于阈值时返回修改建议。

    Args:
        config: Council 配置
        document: 待评审文档全文
        document_type: 文档类型（权利要求书/说明书/审查意见答复）
        threshold: 通过阈值 (0.0-1.0)

    Returns:
        {"passed": bool, "score": float, "issues": [...], "suggestions": [...]}
    """
    task = f"""请作为专利质量评审专家，评审以下{document_type}的质量。

评审维度包括：
1. 形式规范（格式、术语一致性）
2. 实质内容（清楚性、简要性、支持性）
3. 法律合规（是否符合专利法要求）

对每个维度给出：通过/不通过，如果不通过，简要说明问题。

文档：
{document}"""

    result = await run_council(config, task)

    # 分析 Chairman 输出，提取通过/不通过判定
    content = result.stage3.response.content
    passed = "不通过" not in content or content.count("通过") > content.count("不通过")
    score = result._compute_consensus()["score"]

    return {
        "passed": passed and score >= threshold,
        "score": score,
        "threshold": threshold,
        "content": content,
        "aggregate": [
            {"model": a.model, "rank": a.average_rank} for a in result.stage2.aggregate
        ],
    }


# ── CLI ───────────────────────────────────────────────


def main():
    import argparse

    parser = argparse.ArgumentParser(
        description="Patent Council — 专利多模型审议引擎",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
示例:
  python council.py -t "判断权利要求1的创造性" \\
      -m gpt-4o,claude-sonnet-4.5,gemini-2.5-pro \\
      -c claude-sonnet-4.5

  python council.py -t "审查意见答复策略分析" \\
      --gate --threshold 0.8
        """,
    )
    parser.add_argument("-t", "--task", required=True, help="专利分析任务描述")
    parser.add_argument(
        "-m",
        "--models",
        default="gpt-4o,claude-sonnet-4.5,gemini-2.5-pro",
        help="Council 成员模型（逗号分隔）",
    )
    parser.add_argument(
        "-c", "--chairman", default="claude-sonnet-4.5", help="Chairman 模型"
    )
    parser.add_argument(
        "--criteria",
        default="准确性,法律依据,论证深度,完整性",
        help="评审维度（逗号分隔）",
    )
    parser.add_argument("--timeout", type=float, default=120.0)
    parser.add_argument("--max-tokens", type=int, default=4096)
    parser.add_argument("--verbose", action="store_true")
    parser.add_argument("-o", "--output", help="输出 JSON 文件路径")
    parser.add_argument(
        "--gate", action="store_true", help="质量门控模式：文档路径作为 task"
    )
    parser.add_argument("--threshold", type=float, default=0.7, help="门控阈值")

    args = parser.parse_args()

    config = CouncilConfig(
        models=[m.strip() for m in args.models.split(",")],
        chairman=args.chairman.strip(),
        criteria=[c.strip() for c in args.criteria.split(",")],
        timeout=args.timeout,
        max_tokens=args.max_tokens,
        verbose=args.verbose,
    )

    async def _run():
        if args.gate:
            # 质量门控模式：task 是文档路径或直接是文档内容
            task = args.task
            if os.path.isfile(task):
                with open(task) as f:
                    task = f.read()
            result = await quality_gate(config, task, threshold=args.threshold)
            print(json.dumps(result, ensure_ascii=False, indent=2))
        else:
            result = await run_council(config, args.task)
            print(result.summary())
            print()
            if args.output:
                with open(args.output, "w") as f:
                    json.dump(result.to_dict(), f, ensure_ascii=False, indent=2)
                print(f"完整结果已保存至 {args.output}")

    asyncio.run(_run())


if __name__ == "__main__":
    main()
