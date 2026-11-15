import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from app import __version__  # noqa: E402
from app.services import engine, mcp_tools  # noqa: E402

VERSIONS = ["2025-11-25", "2025-06-18", "2025-03-26", "2024-11-05"]


def log(message: str) -> None:
    print(f"[formfabric mcp] {message}", file=sys.stderr, flush=True)


def reply(req_id, result=None, error=None) -> str:
    body = {"jsonrpc": "2.0", "id": req_id}
    if error is not None:
        body["error"] = error
    else:
        body["result"] = result
    return json.dumps(body)


def handle(msg: dict) -> str | None:
    req_id, method, params = msg.get("id"), msg.get("method"), msg.get("params") or {}
    if req_id is None:
        return None
    if method == "initialize":
        asked = params.get("protocolVersion")
        return reply(req_id, {"protocolVersion": asked if asked in VERSIONS else VERSIONS[0], "capabilities": {"tools": {}}, "serverInfo": {"name": "formfabric", "version": __version__}, "instructions": mcp_tools.INSTRUCTIONS})
    if method == "ping":
        return reply(req_id, {})
    if method in ("tools/list", "tools/call") and not engine.available():
        return reply(req_id, error={"code": -32000, "message": engine.UNAVAILABLE})
    if method == "tools/list":
        return reply(req_id, {"tools": mcp_tools.tools()})
    if method == "tools/call":
        name, args = params.get("name"), params.get("arguments") or {}
        if name not in {t["name"] for t in mcp_tools.tools()} or not isinstance(args, dict):
            return reply(req_id, error={"code": -32602, "message": f"No tool named {name}"})
        return reply(req_id, mcp_tools.result_of(engine.dispatch(name, args)))
    return reply(req_id, error={"code": -32601, "message": f"Method not found: {method}"})


def main() -> int:
    sys.stdin.reconfigure(encoding="utf-8", errors="replace")
    sys.stdout.reconfigure(encoding="utf-8", newline="\n")
    if not engine.available():
        log(engine.UNAVAILABLE)
    for line in sys.stdin:
        if not line.strip():
            continue
        try:
            msg = json.loads(line)
        except ValueError:
            out = reply(None, error={"code": -32700, "message": "Parse error"})
        else:
            if not isinstance(msg, dict):
                out = reply(None, error={"code": -32600, "message": "Invalid request"})
            else:
                try:
                    out = handle(msg)
                except Exception as exc:
                    out = reply(msg.get("id"), error={"code": -32603, "message": f"{type(exc).__name__}: {exc}"})
        if out:
            sys.stdout.write(out + "\n")
            sys.stdout.flush()
    return 0


if __name__ == "__main__":
    sys.exit(main())
