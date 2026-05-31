#!/usr/bin/env python3
"""Bridge script for Rust-Python communication via JSON stdin/stdout.

Usage (from Rust):
    echo '{"action":"deliberate","task":"...","config":{...}}' | python3 council_bridge.py

Actions:
    deliberate  — 完整三阶段审议 (stage1/stage2/stage3/consensus)
    quality_gate — 质量门控 (passed/score/issues)
"""
import json
import sys
import os
import asyncio

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from council import CouncilConfig, run_council, quality_gate


async def handle_request(request: dict) -> dict:
    action = request["action"]
    config = CouncilConfig(**request.get("config", {}))
    config.verbose = request.get("verbose", False)

    if action == "deliberate":
        result = await run_council(config, request["task"])
        return result.to_dict()

    if action == "quality_gate":
        result = await quality_gate(
            config,
            request["document"],
            request.get("document_type", "权利要求书"),
            request.get("threshold", 0.7),
        )
        return result

    raise ValueError(f"未知 action: {action}")


if __name__ == "__main__":
    raw = sys.stdin.read()
    try:
        request = json.loads(raw)
    except json.JSONDecodeError as e:
        print(json.dumps({"error": f"JSON 解析错误: {e}"}, ensure_ascii=False))
        sys.exit(1)
    try:
        result = asyncio.run(handle_request(request))
        print(json.dumps(result, ensure_ascii=False))
    except Exception as e:
        print(json.dumps({"error": str(e)}, ensure_ascii=False))
        sys.exit(1)
