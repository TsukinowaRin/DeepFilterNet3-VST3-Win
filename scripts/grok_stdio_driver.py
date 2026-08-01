#!/usr/bin/env python3
"""Run a complete Grok ACP turn over ``grok agent stdio``."""

from __future__ import annotations

import argparse
import io
import json
import os
import pathlib
import selectors
import subprocess
import sys
import tempfile
import time
from typing import Any, BinaryIO, TextIO


ROOT = pathlib.Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / ".agent-shared"))
from hooks_core import evaluate_tool_use  # noqa: E402


class DriverError(RuntimeError):
    pass


class JsonLinesPeer:
    def __init__(self, reader: BinaryIO, writer: BinaryIO, timeout: float) -> None:
        self.reader = reader
        self.writer = writer
        self.deadline = time.monotonic() + timeout
        self.selector = selectors.DefaultSelector()
        self.selector.register(reader, selectors.EVENT_READ)

    def send(self, message: dict[str, Any]) -> None:
        payload = json.dumps(message, ensure_ascii=False, separators=(",", ":"))
        self.writer.write((payload + "\n").encode())
        self.writer.flush()

    def receive(self) -> dict[str, Any]:
        remaining = self.deadline - time.monotonic()
        if remaining <= 0 or not self.selector.select(remaining):
            raise DriverError("timed out waiting for grok agent stdio")
        line = self.reader.readline()
        if not line:
            raise DriverError("grok agent stdio closed stdout unexpectedly")
        try:
            message = json.loads(line)
        except (UnicodeDecodeError, json.JSONDecodeError) as exc:
            raise DriverError(f"invalid JSON-RPC message: {exc}") from exc
        if not isinstance(message, dict):
            raise DriverError("JSON-RPC message must be an object")
        return message


def _tool_details(tool_call: dict[str, Any]) -> tuple[str, str, dict[str, Any]]:
    if "rawInput" in tool_call:
        raw_input = tool_call["rawInput"]
    elif "input" in tool_call:
        raw_input = tool_call["input"]
    else:
        raw_input = {}
    if not isinstance(raw_input, dict):
        return str(tool_call.get("name") or tool_call.get("title") or "unknown"), "malformed", {}
    kind = str(tool_call.get("kind") or "").lower()
    reported_name = str(tool_call.get("name") or tool_call.get("title") or kind or "unknown")
    lowered = reported_name.lower()

    if any(word in lowered or word in kind for word in ("subagent", "agent", "web", "fetch", "memory")):
        return reported_name, "forbidden", raw_input
    if kind in {"read", "search"} or lowered in {"read_file", "view_file", "grep", "search"}:
        hook_name = "grep" if kind == "search" or lowered in {"grep", "search"} else "read_file"
        return hook_name, "file", raw_input
    if kind in {"edit", "write"} or lowered in {
        "edit",
        "write",
        "write_file",
        "search_replace",
        "replace",
    }:
        return "write_file", "file", raw_input
    if kind in {"execute", "shell", "bash"} or lowered in {
        "bash",
        "run_command",
        "run_shell_command",
        "run_terminal_command",
    }:
        return "run_terminal_command", "shell", raw_input
    return reported_name, "unknown", raw_input


def _permission_decision(tool_call: dict[str, Any], allow_bash: bool) -> tuple[bool, str, str]:
    tool_name, category, tool_input = _tool_details(tool_call)
    hook_reason = evaluate_tool_use(tool_name, tool_input)
    if hook_reason:
        return False, tool_name, hook_reason
    if category == "file":
        return True, tool_name, "hooks_core passed file operation"
    if category == "shell":
        if allow_bash:
            return True, tool_name, "hooks_core passed and --allow-bash is set"
        return False, tool_name, "shell tools require --allow-bash"
    if category == "forbidden":
        return False, tool_name, "subagent, web search, and memory tools are disabled"
    if category == "malformed":
        return False, tool_name, "tool input must be an object"
    return False, tool_name, "unknown tool kind"


def _permission_response(params: dict[str, Any], allow_bash: bool) -> dict[str, Any]:
    tool_call = params.get("toolCall") or {}
    if not isinstance(tool_call, dict):
        tool_call = {}
    allowed, tool_name, reason = _permission_decision(tool_call, allow_bash)
    options = params.get("options") or []
    wanted = ("allow_once", "allow_always") if allowed else ("reject_once", "reject_always")
    selected: dict[str, Any] | None = None
    for kind in wanted:
        selected = next(
            (
                option
                for option in options
                if isinstance(option, dict) and option.get("kind") == kind and option.get("optionId")
            ),
            None,
        )
        if selected:
            break
    verdict = "allow" if allowed else "deny"
    print(f"permission tool={tool_name} verdict={verdict} reason={reason}", file=sys.stderr)
    if not selected:
        return {"outcome": {"outcome": "cancelled"}}
    return {"outcome": {"outcome": "selected", "optionId": selected["optionId"]}}


def _raise_rpc_error(message: dict[str, Any]) -> None:
    error = message.get("error")
    if error is not None:
        raise DriverError(f"JSON-RPC error: {error}")


def _wait_for_response(
    peer: JsonLinesPeer,
    request_id: int,
    allow_bash: bool,
    assistant_chunks: list[str],
) -> dict[str, Any]:
    while True:
        message = peer.receive()
        if message.get("method") == "session/request_permission" and "id" in message:
            result = _permission_response(message.get("params") or {}, allow_bash)
            peer.send({"jsonrpc": "2.0", "id": message["id"], "result": result})
            continue
        if message.get("method") == "session/update":
            update = (message.get("params") or {}).get("update") or {}
            if update.get("sessionUpdate") == "agent_message_chunk":
                content = update.get("content") or {}
                if content.get("type") == "text":
                    assistant_chunks.append(str(content.get("text") or ""))
            continue
        if message.get("id") == request_id:
            _raise_rpc_error(message)
            result = message.get("result")
            if not isinstance(result, dict):
                raise DriverError(f"request {request_id} returned a non-object result")
            return result
        if "id" in message and "method" in message:
            peer.send(
                {
                    "jsonrpc": "2.0",
                    "id": message["id"],
                    "error": {"code": -32601, "message": "unsupported agent request"},
                }
            )


def run_driver(
    prompt: str,
    cwd: pathlib.Path,
    model: str | None,
    timeout: float,
    allow_bash: bool,
    agent_command: list[str] | None = None,
) -> str:
    command = agent_command or ["grok", "agent", "stdio"]
    env = os.environ.copy()
    env["PYTHONDONTWRITEBYTECODE"] = "1"
    process = subprocess.Popen(
        command,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=None,
        cwd=cwd,
        env=env,
        bufsize=0,
    )
    assert process.stdin is not None and process.stdout is not None
    peer = JsonLinesPeer(process.stdout, process.stdin, timeout)
    chunks: list[str] = []
    next_id = 1
    try:
        peer.send(
            {
                "jsonrpc": "2.0",
                "id": next_id,
                "method": "initialize",
                "params": {"protocolVersion": 1},
            }
        )
        _wait_for_response(peer, next_id, allow_bash, chunks)
        next_id += 1
        peer.send(
            {
                "jsonrpc": "2.0",
                "id": next_id,
                "method": "session/new",
                "params": {"cwd": str(cwd), "mcpServers": []},
            }
        )
        session = _wait_for_response(peer, next_id, allow_bash, chunks)
        session_id = session.get("sessionId")
        if not session_id:
            raise DriverError("session/new response omitted sessionId")
        if model:
            next_id += 1
            peer.send(
                {
                    "jsonrpc": "2.0",
                    "id": next_id,
                    "method": "session/set_model",
                    "params": {"sessionId": session_id, "modelId": model},
                }
            )
            _wait_for_response(peer, next_id, allow_bash, chunks)
        next_id += 1
        peer.send(
            {
                "jsonrpc": "2.0",
                "id": next_id,
                "method": "session/prompt",
                "params": {
                    "sessionId": session_id,
                    "prompt": [{"type": "text", "text": prompt}],
                },
            }
        )
        _wait_for_response(peer, next_id, allow_bash, chunks)
        return "".join(chunks)
    finally:
        if process.poll() is None:
            process.terminate()
            try:
                process.wait(timeout=2)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait()


def _fake_send(message: dict[str, Any]) -> None:
    sys.stdout.write(json.dumps(message, separators=(",", ":")) + "\n")
    sys.stdout.flush()


def _fake_receive() -> dict[str, Any]:
    line = sys.stdin.readline()
    if not line:
        raise SystemExit("fake agent: client closed stdin")
    return json.loads(line)


def _fake_reply(request: dict[str, Any], result: dict[str, Any]) -> None:
    _fake_send({"jsonrpc": "2.0", "id": request["id"], "result": result})


def run_fake_agent(hang: bool, expect_allow_bash: bool, expect_model: str | None) -> int:
    initialize = _fake_receive()
    if initialize.get("method") != "initialize" or (initialize.get("params") or {}).get(
        "protocolVersion"
    ) != 1:
        raise SystemExit("fake agent: invalid initialize request")
    _fake_reply(initialize, {"protocolVersion": 1})
    new_session = _fake_receive()
    if new_session.get("method") != "session/new" or new_session.get("params") != {
        "cwd": os.getcwd(),
        "mcpServers": [],
    }:
        raise SystemExit("fake agent: invalid session/new request")
    _fake_reply(new_session, {"sessionId": "selftest-session"})
    if expect_model:
        set_model = _fake_receive()
        if set_model.get("method") != "session/set_model" or set_model.get("params") != {
            "sessionId": "selftest-session",
            "modelId": expect_model,
        }:
            raise SystemExit("fake agent: invalid session/set_model request")
        _fake_reply(set_model, {})
    prompt = _fake_receive()
    prompt_params = prompt.get("params") or {}
    if (
        prompt.get("method") != "session/prompt"
        or prompt_params.get("sessionId") != "selftest-session"
        or prompt_params.get("prompt") != [{"type": "text", "text": "selftest prompt"}]
    ):
        raise SystemExit("fake agent: invalid session/prompt request")
    if hang:
        time.sleep(60)
        return 0

    cases = [
        ({"kind": "edit", "name": "search_replace", "rawInput": {"path": "probe.txt"}}, True),
        ({"kind": "execute", "name": "run_terminal_command", "rawInput": {"command": "git reset --hard HEAD"}}, False),
        (
            {"kind": "execute", "name": "run_terminal_command", "rawInput": {"command": "printf ok"}},
            expect_allow_bash,
        ),
        ({"kind": "read", "name": "read_file", "rawInput": {"path": ".env"}}, False),
        ({"kind": "fetch", "name": "web_search", "rawInput": {"query": "x"}}, False),
        ({"kind": "other", "name": "subagent", "rawInput": {}}, False),
        ({"kind": "other", "name": "mystery", "rawInput": {}}, False),
        ({"kind": "edit", "name": "search_replace", "rawInput": []}, False),
    ]
    options = [
        {"optionId": "yes", "kind": "allow_once", "name": "Allow"},
        {"optionId": "no", "kind": "reject_once", "name": "Reject"},
    ]
    for index, (tool_call, expected_allow) in enumerate(cases, start=100):
        _fake_send(
            {
                "jsonrpc": "2.0",
                "id": index,
                "method": "session/request_permission",
                "params": {"sessionId": "selftest-session", "toolCall": tool_call, "options": options},
            }
        )
        response = _fake_receive()
        selected = ((response.get("result") or {}).get("outcome") or {}).get("optionId")
        if selected != ("yes" if expected_allow else "no"):
            raise SystemExit(f"fake agent: wrong permission response for {tool_call['name']}")

    for text in ("SELFTEST: ", "PASS"):
        _fake_send(
            {
                "jsonrpc": "2.0",
                "method": "session/update",
                "params": {
                    "sessionId": "selftest-session",
                    "update": {
                        "sessionUpdate": "agent_message_chunk",
                        "content": {"type": "text", "text": text},
                    },
                },
            }
        )
    _fake_reply(prompt, {"stopReason": "end_turn"})
    return 0


def run_selftest() -> int:
    script = str(pathlib.Path(__file__).resolve())
    with tempfile.TemporaryDirectory(prefix="grok-stdio-selftest-") as fixture:
        command = [sys.executable, script, "--fake-agent"]
        output = run_driver("selftest prompt", pathlib.Path(fixture), None, 5, False, command)
        if output != "SELFTEST: PASS":
            raise DriverError(f"selftest text mismatch: {output!r}")
        bash_command = [
            sys.executable,
            script,
            "--fake-agent",
            "--fake-expect-allow-bash",
            "--fake-expect-model",
            "selftest-model",
        ]
        output = run_driver(
            "selftest prompt",
            pathlib.Path(fixture),
            "selftest-model",
            5,
            True,
            bash_command,
        )
        if output != "SELFTEST: PASS":
            raise DriverError(f"selftest allow-bash/model text mismatch: {output!r}")
        argv_args = parse_args(["first", "second"])
        if _resolve_prompt(argv_args, sys.stdin) != "first second":
            raise DriverError("selftest argv prompt joining failed")
        stdin_args = parse_args(["--prompt-stdin"])
        if _resolve_prompt(stdin_args, io.StringIO("stdin prompt")) != "stdin prompt":
            raise DriverError("selftest stdin prompt failed")
        hang_command = [sys.executable, script, "--fake-agent", "--fake-hang"]
        try:
            run_driver("timeout prompt", pathlib.Path(fixture), None, 0.1, False, hang_command)
        except DriverError as exc:
            if "timed out" not in str(exc):
                raise
        else:
            raise DriverError("selftest expected timeout")
    print("grok stdio driver selftest: PASS")
    return 0


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cwd", type=pathlib.Path)
    parser.add_argument("--model")
    parser.add_argument("--timeout", type=float, default=1800)
    parser.add_argument("--allow-bash", action="store_true")
    parser.add_argument("--prompt-stdin", action="store_true")
    parser.add_argument("--selftest", action="store_true")
    parser.add_argument("--fake-agent", action="store_true", help=argparse.SUPPRESS)
    parser.add_argument("--fake-hang", action="store_true", help=argparse.SUPPRESS)
    parser.add_argument("--fake-expect-allow-bash", action="store_true", help=argparse.SUPPRESS)
    parser.add_argument("--fake-expect-model", help=argparse.SUPPRESS)
    parser.add_argument("prompt", nargs="*")
    return parser.parse_args(argv)


def _resolve_prompt(args: argparse.Namespace, stream: TextIO) -> str:
    if args.prompt_stdin:
        if args.prompt:
            raise DriverError("prompt argv cannot be combined with --prompt-stdin")
        prompt = stream.read()
    else:
        prompt = " ".join(args.prompt)
    if not prompt:
        raise DriverError("prompt must not be empty")
    return prompt


def main(argv: list[str] | None = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    if args.fake_agent:
        return run_fake_agent(args.fake_hang, args.fake_expect_allow_bash, args.fake_expect_model)
    try:
        if args.selftest:
            return run_selftest()
        if args.cwd is None or not args.cwd.is_absolute():
            raise DriverError("--cwd must be an absolute path")
        if args.timeout <= 0:
            raise DriverError("--timeout must be greater than zero")
        prompt = _resolve_prompt(args, sys.stdin)
        output = run_driver(prompt, args.cwd, args.model, args.timeout, args.allow_bash)
        sys.stdout.write(output)
        if output and not output.endswith("\n"):
            sys.stdout.write("\n")
        return 0
    except (DriverError, OSError) as exc:
        print(f"grok stdio driver: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
