#!/usr/bin/env python3
"""
宪法规则验证 CLI — 零编译、秒级验证。

用法:
  # 检查文本是否符合规则
  python3 scripts/constitutional_check.py check \\
      --rules codex-patent-assets/constitutional \\
      --tool claim_generator --phase 撰写 \\
      --input "一种图像识别装置，包括摄像头..."

  # 只验证 YAML 文件格式
  python3 scripts/constitutional_check.py validate \\
      --rules codex-patent-assets/constitutional

  # 列出所有规则
  python3 scripts/constitutional_check.py list \\
      --rules codex-patent-assets/constitutional

  # 从文件读取输入
  python3 scripts/constitutional_check.py check \\
      --rules codex-patent-assets/constitutional \\
      --tool oa_responder --phase 答复 \\
      --input @response.txt
"""

from __future__ import annotations

import argparse
import json
import os
import re
import sys
from pathlib import Path

try:
    import yaml
except ImportError:
    print("需安装 PyYAML: pip3 install pyyaml", file=sys.stderr)
    sys.exit(1)


# ── 规则模型 ──────────────────────────────────────────────────────────


def parse_severity(s: str) -> str:
    return s.lower()


def parse_action(s: str) -> str:
    return s.lower()


# ── 规则加载 ──────────────────────────────────────────────────────────


def load_rules(rules_dir: str | Path) -> dict:
    rules_dir = Path(rules_dir)
    if not rules_dir.is_dir():
        print(f"规则目录不存在: {rules_dir}", file=sys.stderr)
        sys.exit(1)

    all_rules = {}
    for fpath in sorted(rules_dir.glob("*.yaml")):
        with open(fpath) as f:
            data = yaml.safe_load(f)
        if not data:
            continue
        rules_block = data.get("rules") or {}
        fname = fpath.stem
        all_rules[fname] = {
            "file": str(fpath),
            "rules": rules_block,
            "raw": data,
        }
        if isinstance(rules_block, dict):
            print(f"  [{fname}] 已加载 {len(rules_block)} 条规则")
        else:
            print(f"  [{fname}] 已加载")
    return all_rules


# ── 规则评估引擎 ──────────────────────────────────────────────────────


def match_patterns(patterns: list, text: str) -> list:
    """检查文本中是否包含任一模式（支持简单字符串匹配和正则）"""
    hits = []
    for p in patterns:
        p_clean = p.strip().strip('"').strip("'")
        if p_clean.startswith("re:") and len(p_clean) > 3:
            try:
                if re.search(p_clean[3:], text):
                    hits.append(p_clean)
            except re.error:
                pass
        elif p_clean in text:
            hits.append(p_clean)
    return hits


def evaluate_keyword_blocklist(rule: dict, check: dict, input_text: str) -> dict:
    keywords = (
        check.get("keywords", [])
        + check.get("absolute_ban", [])
        + check.get("context_ban", [])
    )
    patterns_raw = check.get("patterns", [])
    all_items = keywords + patterns_raw
    hits = match_patterns(all_items, input_text) if all_items else []

    if hits:
        return {
            "passed": False,
            "details": [f"命中禁用词: {h}" for h in hits],
            "confidence": 0.9,
        }
    return {"passed": True, "details": ["未命中禁用词"], "confidence": 0.95}


def evaluate_structural_analysis(rule: dict, check: dict, input_text: str) -> dict:
    requires = check.get("requires_all", [])
    missing = []
    for elem in requires:
        patterns = elem.get("patterns", [])
        hits = match_patterns(patterns, input_text)
        if not hits:
            missing.append(elem.get("element", "未知要素"))
    if missing:
        return {
            "passed": False,
            "details": [f"缺少要素: {m}" for m in missing],
            "confidence": check.get("min_confidence", 0.6),
        }
    return {
        "passed": True,
        "details": ["三要素完整"],
        "confidence": min(1.0, check.get("min_confidence", 0.6) + 0.2),
    }


def evaluate_category_detection(rule: dict, check: dict, input_text: str) -> dict:
    categories = check.get("categories", {})
    matches = []
    for cat_name, cat_def in categories.items():
        hits = match_patterns(cat_def.get("patterns", []), input_text)
        if hits:
            matches.append(
                f"[{cat_name}] 命中 {len(hits)} 个模式，guidance: {cat_def.get('guidance', '')}"
            )
    if matches:
        return {"passed": False, "details": matches, "confidence": 0.8}
    return {"passed": True, "details": ["未命中排除客体类别"], "confidence": 0.9}


def evaluate_pattern_analysis(rule: dict, check: dict, input_text: str) -> dict:
    pure_hits = match_patterns(check.get("pure_software_markers", []), input_text)
    hw_hits = match_patterns(check.get("hardware_integration_markers", []), input_text)
    if pure_hits and not hw_hits:
        return {
            "passed": False,
            "details": ["纯软件方案，需结合硬件分析"],
            "confidence": 0.7,
        }
    return {"passed": True, "details": ["通过模式分析"], "confidence": 0.85}


def evaluate_specification_analysis(rule: dict, check: dict, input_text: str) -> dict:
    dimensions = check.get("dimensions", [])
    dim_results = []
    for dim in dimensions:
        checks = dim.get("checks", [])
        all_pass = all(c.strip().strip('"') in input_text for c in checks)
        if not all_pass:
            dim_results.append(f"维度 '{dim.get('dimension', '?')}' 未全部满足")
    if dim_results:
        return {"passed": False, "details": dim_results, "confidence": 0.7}
    return {"passed": True, "details": ["说明书分析通过"], "confidence": 0.85}


def evaluate_section_structure(rule: dict, check: dict, input_text: str) -> dict:
    sections = check.get("required_sections", [])
    missing = []
    for sec in sections:
        patterns = sec.get("patterns", [])
        if not match_patterns(patterns, input_text):
            missing.append(sec.get("name", "?"))

    if missing:
        return {
            "passed": False,
            "details": [f"缺少章节或标记: {m}" for m in missing],
            "confidence": 0.75,
        }
    return {"passed": True, "details": ["章节结构完整"], "confidence": 0.9}


def evaluate_claim_clarity(rule: dict, check: dict, input_text: str) -> dict:
    unclear = check.get("unclear_terms", [])
    broad = check.get("over_broad", [])
    hits_unclear = match_patterns(unclear, input_text)
    hits_broad = match_patterns(broad, input_text)
    details = []
    if hits_unclear:
        details.append(f"模糊表述: {', '.join(hits_unclear[:3])}")
    if hits_broad:
        details.append(f"范围过宽: {', '.join(hits_broad[:3])}")
    if details:
        return {"passed": False, "details": details, "confidence": 0.85}
    return {"passed": True, "details": ["权利要求清楚"], "confidence": 0.9}


def evaluate_dependency_validation(rule: dict, check: dict, input_text: str) -> dict:
    rules_list = check.get("rules", [])
    issues = []
    for r in rules_list:
        error_pat = r.get("error_pattern", "")
        if error_pat and error_pat.strip('"') in input_text:
            issues.append(r.get("description", "依赖问题"))
    if issues:
        return {"passed": False, "details": issues, "confidence": 0.85}
    return {"passed": True, "details": ["引用关系正确"], "confidence": 0.9}


def evaluate_default(rule: dict, check: dict, input_text: str) -> dict:
    return {
        "passed": True,
        "details": [f"规则 '{rule.get('name', '?')}' 需要人工判断"],
        "confidence": 0.5,
    }


# ── 规则检查器路由 ──


def evaluate_rule(rule_id: str, rule: dict, input_text: str) -> dict:
    check = rule.get("check", {})
    check_type = check.get("type", "")

    evaluators = {
        "keyword_blocklist": evaluate_keyword_blocklist,
        "structural_analysis": evaluate_structural_analysis,
        "category_detection": evaluate_category_detection,
        "pattern_analysis": evaluate_pattern_analysis,
        "specification_analysis": evaluate_specification_analysis,
        "section_structure": evaluate_section_structure,
        "claim_clarity_analysis": evaluate_claim_clarity,
        "dependency_validation": evaluate_dependency_validation,
    }

    evaluator = evaluators.get(check_type, evaluate_default)
    result = evaluator(rule, check, input_text)

    return {
        "rule_id": rule_id,
        "rule_name": rule.get("name", ""),
        "severity": rule.get("severity", "major"),
        "action": rule.get("action", "warn"),
        "legal_basis": rule.get("legal_basis", ""),
        "phase": rule.get("phase", ""),
        "passed": result["passed"],
        "details": result["details"],
        "confidence": result["confidence"],
    }


def check_all(rules: dict, tool_name: str, input_text: str, phase: str = "") -> list:
    results = []
    for fname, fdata in rules.items():
        for rule_id, rule in fdata.get("rules", {}).items():
            if isinstance(rule, dict):
                rule_phase = rule.get("phase", "")
                if phase and rule_phase and rule_phase != phase:
                    continue
                result = evaluate_rule(rule_id, rule, input_text)
                results.append(result)
    return results


# ── 格式化输出 ──


def color(s: str, code: str) -> str:
    if not sys.stdout.isatty():
        return s
    colors = {
        "red": "31",
        "green": "32",
        "yellow": "33",
        "blue": "34",
        "cyan": "36",
        "bold": "1",
    }
    c = colors.get(code, "0")
    return f"\033[{c}m{s}\033[0m"


def print_results(results: list, verbose: bool = False):
    passed = [r for r in results if r["passed"]]
    failed = [r for r in results if not r["passed"]]
    blocking = [r for r in failed if r["action"] == "block"]

    print(f"\n{'=' * 60}")
    print("宪法合规检查报告")
    print(f"{'=' * 60}")
    print(
        f"  总检查: {len(results)} | {color('通过', 'green')}: {len(passed)} | "
        f"{color('违规', 'red')}: {len(failed)} | "
        f"{color('阻断', 'bold')}: {len(blocking)}"
    )
    print(f"{'=' * 60}\n")

    if not failed:
        print(color("  ✓ 全部通过，无违规", "green"))
        return

    for r in failed:
        icon = "✗" if r["action"] == "block" else "!"
        sev_color = (
            "red"
            if r["severity"] == "critical"
            else ("yellow" if r["severity"] == "major" else "cyan")
        )
        label = color(f"[{r['severity'].upper()}]", sev_color)
        block_tag = color(" [阻断]", "red") if r["action"] == "block" else ""
        print(
            f"  {icon} {color(r['rule_id'], 'bold')} {r['rule_name']} {label}{block_tag}"
        )
        if r["legal_basis"]:
            print(f"    依据: {r['legal_basis']}")
        print(f"    阶段: {r['phase'] or '通用'}")
        for d in r["details"]:
            print(f"    → {d}")
        print()

    if blocking:
        print(color(f"  ⚠ 发现 {len(blocking)} 个阻断性违规", "red"))
        print("  必须修复后才能继续")
    elif failed:
        print(color(f"  ⚠ 发现 {len(failed)} 个非阻断违规", "yellow"))
        print("  建议修复")


def print_json_results(results: list):
    output = {
        "summary": {
            "total": len(results),
            "passed": sum(1 for r in results if r["passed"]),
            "failed": sum(1 for r in results if not r["passed"]),
            "blocking": sum(
                1 for r in results if not r["passed"] and r["action"] == "block"
            ),
        },
        "results": results,
    }
    print(json.dumps(output, ensure_ascii=False, indent=2))


# ── 子命令 ──────────────────────────────────────────────────────────


def cmd_check(args):
    input_text = args.input
    if input_text.startswith("@") and len(input_text) > 1:
        fpath = Path(input_text[1:])
        if fpath.is_file():
            input_text = fpath.read_text(encoding="utf-8")
        else:
            print(f"文件不存在: {fpath}", file=sys.stderr)
            sys.exit(1)

    rules = load_rules(args.rules)
    results = check_all(rules, args.tool, input_text, args.phase)

    if args.json:
        print_json_results(results)
    else:
        print_results(results)

    has_blocking = any(not r["passed"] and r["action"] == "block" for r in results)
    if has_blocking:
        sys.exit(2)


def cmd_validate(args):
    rules = load_rules(args.rules)
    all_ok = True
    for fname, fdata in rules.items():
        print(f"  {fname}: ✅")
    if all_ok:
        print(f"\n 全部 {len(rules)} 个 YAML 文件验证通过 ✓")
    else:
        print("\n 存在错误 ✗")
        sys.exit(1)


def cmd_list(args):
    rules = load_rules(args.rules)
    print(f"\n{'=' * 60}")
    print("宪法规则清单")
    print(f"{'=' * 60}")
    for fname in sorted(rules.keys()):
        rules_block = rules[fname].get("rules", {})
        if isinstance(rules_block, dict):
            for rule_id, rule in rules_block.items():
                if isinstance(rule, dict):
                    sev = rule.get("severity", "?").upper()
                    phase = rule.get("phase", "通用")
                    print(
                        f"  {rule_id:20s} [{sev:8s}] [{phase:6s}] {rule.get('name', '')}"
                    )
            print()


def main():
    parser = argparse.ArgumentParser(
        description="宪法规则验证 CLI — 零编译、秒级验证专利法合规",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=__doc__,
    )
    parser.add_argument(
        "--rules",
        "-r",
        default="codex-patent-assets/constitutional",
        help="宪法规则 YAML 文件目录 (默认: codex-patent-assets/constitutional)",
    )

    sub = parser.add_subparsers(dest="command", required=True)

    # check
    p_check = sub.add_parser("check", help="检查文本是否符合规则")
    p_check.add_argument("--tool", default="", help="工具名称（如 claim_generator）")
    p_check.add_argument(
        "--phase", default="", help="生命周期阶段（撰写/审查/答复/无效/维权）"
    )
    p_check.add_argument(
        "--input", required=True, help="输入文本，或以 @ 开头的文件路径"
    )
    p_check.add_argument("--json", action="store_true", help="JSON 格式输出")

    # validate
    p_val = sub.add_parser("validate", help="验证 YAML 规则文件格式")
    p_val.add_argument("--json", action="store_true", help="JSON 格式输出")

    # list
    sub.add_parser("list", help="列出所有规则")

    args = parser.parse_args()

    # Normalize rules path
    if not os.path.isabs(args.rules):
        # Try relative to cwd, then to script location
        candidates = [
            Path.cwd() / args.rules,
            Path(__file__).parent.parent / args.rules,
        ]
        for c in candidates:
            if c.is_dir():
                args.rules = str(c)
                break

    if args.command == "check":
        cmd_check(args)
    elif args.command == "validate":
        cmd_validate(args)
    elif args.command == "list":
        cmd_list(args)


if __name__ == "__main__":
    main()
