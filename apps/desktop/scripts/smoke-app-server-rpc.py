#!/usr/bin/env python3
"""M4 冒烟：经 stdio 验证 app-server JSON-RPC 握手与 thread/start（可选 turn/start）。"""
from __future__ import annotations

import json
import os
import select
import shutil
import subprocess
import sys
import time

BCIP = os.environ.get("BCIP", "bcip")
SKIP_TURN = os.environ.get("BCIP_SMOKE_SKIP_TURN", "1") == "1"
TURN_TIMEOUT_SEC = int(os.environ.get("BCIP_SMOKE_TURN_TIMEOUT", "45"))


def eprint(*args: object) -> None:
    print(*args, file=sys.stderr)


def read_json_line(proc: subprocess.Popen[str], timeout: float = 120.0) -> dict:
    deadline = time.time() + timeout
    while time.time() < deadline:
        if proc.poll() is not None:
            raise RuntimeError(f"bcip 已退出 code={proc.returncode}")
        ready, _, _ = select.select([proc.stdout], [], [], 0.2)
        if not ready:
            continue
        line = proc.stdout.readline()
        if not line:
            continue
        line = line.strip()
        if not line:
            continue
        try:
            return json.loads(line)
        except json.JSONDecodeError as err:
            eprint(f"warn: 非 JSON 行已忽略: {line[:200]!r} ({err})")
    raise TimeoutError("等待 app-server 响应超时")


def wait_for_result(proc: subprocess.Popen[str], req_id: int, timeout: float = 120.0) -> dict:
    deadline = time.time() + timeout
    while time.time() < deadline:
        msg = read_json_line(proc, timeout=max(0.5, deadline - time.time()))
        if msg.get("id") == req_id:
            if "error" in msg:
                raise RuntimeError(
                    f"RPC 错误 id={req_id}: {msg['error'].get('message', msg['error'])}"
                )
            return msg.get("result") or {}
    raise TimeoutError(f"未收到 id={req_id} 的响应")


def send_request(proc: subprocess.Popen[str], req_id: int, method: str, params: object) -> dict:
    payload = {"jsonrpc": "2.0", "id": req_id, "method": method, "params": params}
    proc.stdin.write(json.dumps(payload, ensure_ascii=False) + "\n")
    proc.stdin.flush()
    return wait_for_result(proc, req_id)


def send_notification(proc: subprocess.Popen[str], method: str, params: object | None = None) -> None:
    payload: dict = {"jsonrpc": "2.0", "method": method}
    if params is not None:
        payload["params"] = params
    proc.stdin.write(json.dumps(payload, ensure_ascii=False) + "\n")
    proc.stdin.flush()


def drain_until_turn_done(proc: subprocess.Popen[str], timeout: float) -> None:
    deadline = time.time() + timeout
    while time.time() < deadline:
        msg = read_json_line(proc, timeout=max(0.5, deadline - time.time()))
        method = msg.get("method")
        if method == "turn/completed":
            eprint("ok: 收到 turn/completed")
            return
        if method == "error" or (
            isinstance(msg.get("params"), dict)
            and msg.get("params", {}).get("error")
        ):
            eprint(f"warn: 通知 {method} {msg.get('params')}")
    raise TimeoutError("turn/start 未在时限内完成（可设 BCIP_SMOKE_SKIP_TURN=1 跳过）")


def main() -> int:
    if not shutil.which(BCIP):
        eprint(f"skip: 未找到 {BCIP}，跳过 RPC 冒烟（设置 BCIP_SMOKE_REQUIRE=1 可强制失败）")
        if os.environ.get("BCIP_SMOKE_REQUIRE") == "1":
            return 1
        return 0

    eprint(f"==> app-server stdio 冒烟 ({BCIP})")
    proc = subprocess.Popen(
        [BCIP, "app-server", "--listen", "stdio://"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        bufsize=1,
    )
    assert proc.stdin and proc.stdout

    try:
        init = send_request(
            proc,
            1,
            "initialize",
            {
                "clientInfo": {
                    "name": "bcip-desktop-smoke",
                    "title": "BCIP Desktop Smoke",
                    "version": "0.0.0",
                }
            },
        )
        if "codexHome" not in init and "userAgent" not in init:
            eprint(f"warn: initialize 结果异常: {init}")
        send_notification(proc, "initialized")
        eprint("ok: initialize + initialized")

        thread = send_request(proc, 2, "thread/start", {"cwd": None})
        thread_id = (thread.get("thread") or {}).get("id")
        if not thread_id:
            raise RuntimeError(f"thread/start 无 thread.id: {thread}")
        eprint(f"ok: thread/start → {thread_id}")

        if SKIP_TURN:
            eprint("ok: 已跳过 turn/start（BCIP_SMOKE_SKIP_TURN=1）")
        else:
            send_request(
                proc,
                3,
                "turn/start",
                {
                    "threadId": thread_id,
                    "input": [{"type": "text", "text": "smoke ping", "text_elements": []}],
                },
            )
            eprint("ok: turn/start 已发送，等待完成…")
            drain_until_turn_done(proc, TURN_TIMEOUT_SEC)

        eprint("ok: app-server RPC 冒烟通过")
        return 0
    finally:
        proc.stdin.close()
        try:
            proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            proc.kill()

    return 0


if __name__ == "__main__":
    sys.exit(main())
