import json

from fastapi import APIRouter, Depends, Request, Response
from pydantic import BaseModel
from starlette.concurrency import run_in_threadpool

from app import __version__
from app.core import window_guard
from app.services import agents, engine, events, mcp_tools

router = APIRouter()
VERSIONS = ["2026-07-28", "2025-11-25"]
CACHE_HINTS = {"ttlMs": 300_000, "cacheScope": "public"}
UNBUILT = "The formfabric engine module is not built; run ./formfabric engine build in the formfabric folder."


def _local_origin(origin: str | None) -> bool:
    if not origin:
        return True
    return window_guard.exact(origin) in window_guard.window_origins()


def _ok(req_id, result: dict, modern: bool, cacheable: bool = False) -> dict:
    extra = {"resultType": "complete", **(CACHE_HINTS if cacheable else {})} if modern else {}
    return {"jsonrpc": "2.0", "id": req_id, "result": {**result, **extra}}


def _err(req_id, code: int, message: str) -> dict:
    return {"jsonrpc": "2.0", "id": req_id, "error": {"code": code, "message": message}}


async def call_tool(name: str, args: dict, inapp: bool = False) -> dict:
    refusal = agents.switched_off()
    if refusal is None and not engine.read_only(name):
        refusal = await agents.permit(name, inapp) or agents.switched_off()
    if refusal:
        return {"content": [{"type": "text", "text": refusal}], "isError": True}
    reply = await run_in_threadpool(engine.dispatch, name, args)
    return mcp_tools.result_of(reply)


@router.post("/mcp")
async def mcp(request: Request) -> Response:
    if not _local_origin(request.headers.get("origin")):
        return Response(status_code=403, content=json.dumps(_err(None, -32600, "Origin not allowed")), media_type="application/json")
    try:
        msg = json.loads(await request.body())
    except ValueError:
        return Response(content=json.dumps(_err(None, -32700, "Parse error")), media_type="application/json", status_code=400)
    if isinstance(msg, list):
        return Response(content=json.dumps(_err(None, -32600, "Batches are not supported")), media_type="application/json", status_code=400)
    if not isinstance(msg, dict):
        return Response(content=json.dumps(_err(None, -32600, "Invalid request")), media_type="application/json", status_code=400)
    req_id, method, params = msg.get("id"), msg.get("method"), msg.get("params") or {}
    if req_id is None:
        return Response(status_code=202)
    if not isinstance(params, dict) or not isinstance(params.get("_meta") or {}, dict) or not isinstance(params.get("name") or "", str):
        return Response(content=json.dumps(_err(req_id, -32602, "Invalid params")), media_type="application/json", status_code=400)
    meta = params.get("_meta") or {}
    version = meta.get("io.modelcontextprotocol/protocolVersion") or request.headers.get("mcp-protocol-version") or "2025-11-25"
    modern = version == "2026-07-28"
    server = {"name": "formfabric", "version": __version__}
    if not engine.available() and method in ("tools/list", "tools/call"):
        out = _err(req_id, -32000, UNBUILT)
    elif version not in VERSIONS and method != "initialize":
        out = _err(req_id, -32602, f"Unsupported protocol version {version}; supported: {', '.join(VERSIONS)}")
    elif method == "server/discover":
        out = _ok(req_id, {"supportedVersions": VERSIONS, "capabilities": {"tools": {}}, "_meta": {"io.modelcontextprotocol/serverInfo": server}, "instructions": mcp_tools.INSTRUCTIONS}, True, cacheable=True)
    elif method == "initialize":
        asked = params.get("protocolVersion")
        out = _ok(req_id, {"protocolVersion": asked if asked in VERSIONS else "2025-11-25", "capabilities": {"tools": {}}, "serverInfo": server, "instructions": mcp_tools.INSTRUCTIONS}, False)
    elif method == "ping":
        out = _ok(req_id, {}, modern)
    elif method == "tools/list":
        out = _ok(req_id, {"tools": mcp_tools.tools()}, modern, cacheable=True)
    elif method == "tools/call":
        name, args = params.get("name"), params.get("arguments") or {}
        names = {t["name"] for t in mcp_tools.tools()}
        inapp = agents.is_inapp(request.headers.get(agents.INAPP_HEADER, ""))
        if inapp:
            events.publish("agent.activity", tool=name)
        out = _ok(req_id, await call_tool(name, args, inapp), modern) if name in names and isinstance(args, dict) else _err(req_id, -32602, f"No tool named {name}")
    else:
        out = _err(req_id, -32601, f"Method not found: {method}")
    return Response(content=json.dumps(out), media_type="application/json")


class Consent(BaseModel):
    allow: bool


@router.get("/mcp/status")
def status() -> dict:
    return {"endpoint": "/api/mcp", "engine": engine.available(), "tools": len(mcp_tools.tools()) if engine.available() else 0, **agents.status()}


@router.get("/mcp/window", dependencies=[Depends(window_guard.require_window_read)])
def window(since: int = -1) -> dict:
    return {"epoch": engine.epoch(), "changed": engine.epoch() != since, "consent": agents.pending()}


@router.post("/mcp/window", dependencies=[Depends(window_guard.require_window)])
def window_poll(since: int = -1) -> dict:
    agents.seen()
    return window(since)


@router.post("/mcp/consent", dependencies=[Depends(window_guard.require_window)])
async def consent(body: Consent) -> dict:
    agents.answer(body.allow)
    return agents.status()


@router.post("/mcp/reset", dependencies=[Depends(window_guard.require_window)])
def reset() -> dict:
    agents.reset()
    return agents.status()
