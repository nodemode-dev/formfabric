import json
import os
import subprocess
import sys
from pathlib import Path

import pytest

BACKEND = Path(__file__).resolve().parents[2]
VERSION = "0.31.0"


def well_formed(body, req_id):
    assert body["jsonrpc"] == "2.0"
    assert body["id"] == req_id
    assert ("result" in body) != ("error" in body)
    if "error" in body:
        assert isinstance(body["error"]["code"], int) and not isinstance(body["error"]["code"], bool)
        assert isinstance(body["error"]["message"], str) and body["error"]["message"]
        assert set(body["error"]) <= {"code", "message", "data"}
    assert set(body) <= {"jsonrpc", "id", "result", "error"}
    return body


def rpc(bare, method, params=None, req_id=1, headers=None):
    msg = {"jsonrpc": "2.0", "id": req_id, "method": method}
    if params is not None:
        msg["params"] = params
    r = bare.post("/api/mcp", json=msg, headers=headers or {})
    assert r.headers["content-type"].startswith("application/json")
    return well_formed(r.json(), req_id)


def engine_absent():
    from app.services import engine

    return not engine.available()


def test_text_that_is_not_json_gets_the_parse_error_code_with_a_null_id(bare):
    r = bare.post("/api/mcp", content=b"{not json", headers={"content-type": "application/json"})
    body = well_formed(r.json(), None)
    assert body["error"]["code"] == -32700


def test_a_batch_is_answered_with_invalid_request(bare):
    r = bare.post("/api/mcp", json=[{"jsonrpc": "2.0", "id": 1, "method": "ping"}])
    assert well_formed(r.json(), None)["error"]["code"] == -32600


@pytest.mark.parametrize("payload", ["ping", 7, True])
def test_a_json_value_that_is_not_an_object_is_answered_with_invalid_request(bare, payload):
    r = bare.post("/api/mcp", json=payload)
    assert well_formed(r.json(), None)["error"]["code"] == -32600


def test_a_notification_gets_no_reply_body(bare):
    r = bare.post("/api/mcp", json={"jsonrpc": "2.0", "method": "notifications/initialized"})
    assert r.status_code == 202
    assert r.content == b""


@pytest.mark.parametrize("req_id", [1, 0, 99999, "abc", "7"])
def test_ping_answers_an_empty_result_under_the_same_id_and_type(bare, req_id):
    body = rpc(bare, "ping", req_id=req_id)
    assert body["result"] == {}
    assert type(body["id"]) is type(req_id)


def test_an_unknown_method_gets_method_not_found_naming_it(bare):
    body = rpc(bare, "resources/teleport", req_id=5)
    assert body["error"]["code"] == -32601
    assert "resources/teleport" in body["error"]["message"]


@pytest.mark.parametrize("params", [[1, 2], "text", 5])
def test_params_that_are_not_an_object_get_invalid_params(bare, params):
    body = rpc(bare, "tools/call", params=params, req_id=3)
    assert body["error"]["code"] == -32602


@pytest.mark.parametrize("asked", ["2026-07-28", "2025-11-25"])
def test_initialize_echoes_a_protocol_version_it_supports(bare, asked):
    result = rpc(bare, "initialize", {"protocolVersion": asked, "capabilities": {}, "clientInfo": {"name": "t", "version": "0"}})["result"]
    assert result["protocolVersion"] == asked


def test_initialize_answers_an_unsupported_version_with_one_it_does_support(bare):
    result = rpc(bare, "initialize", {"protocolVersion": "1999-01-01", "capabilities": {}})["result"]
    discovered = rpc(bare, "server/discover")["result"]["supportedVersions"]
    assert result["protocolVersion"] != "1999-01-01"
    assert result["protocolVersion"] in discovered


def test_initialize_names_the_server_its_version_and_the_tools_capability(bare):
    result = rpc(bare, "initialize", {"protocolVersion": "2025-11-25", "capabilities": {}})["result"]
    assert result["serverInfo"] == {"name": "formfabric", "version": VERSION}
    assert isinstance(result["capabilities"]["tools"], dict)
    assert "millimetres" in result["instructions"]


def test_the_server_version_matches_the_health_route(bare, client):
    result = rpc(bare, "initialize", {"protocolVersion": "2025-11-25"})["result"]
    health = client.get("/api/health")
    if health.status_code != 200 or "version" not in health.json():
        pytest.skip("this build has no health route reporting a version to compare with")
    assert result["serverInfo"]["version"] == health.json()["version"]


def test_a_request_under_an_unsupported_protocol_version_is_refused_with_the_supported_list(bare):
    body = rpc(bare, "ping", headers={"mcp-protocol-version": "1999-01-01"})
    assert body["error"]["code"] == -32602
    for version in rpc(bare, "server/discover")["result"]["supportedVersions"]:
        assert version in body["error"]["message"]
    assert "1999-01-01" in body["error"]["message"]


def test_the_newer_protocol_version_marks_results_complete_and_the_older_one_does_not(bare):
    modern = rpc(bare, "ping", headers={"mcp-protocol-version": "2026-07-28"})["result"]
    older = rpc(bare, "ping", headers={"mcp-protocol-version": "2025-11-25"})["result"]
    assert modern == {"resultType": "complete"}
    assert older == {}


def test_the_protocol_version_in_request_meta_wins_over_the_header(bare):
    result = rpc(bare, "ping", {"_meta": {"io.modelcontextprotocol/protocolVersion": "2026-07-28"}}, headers={"mcp-protocol-version": "2025-11-25"})["result"]
    assert result.get("resultType") == "complete"


def test_a_request_from_a_foreign_web_origin_is_forbidden(bare):
    r = bare.post("/api/mcp", json={"jsonrpc": "2.0", "id": 1, "method": "ping"}, headers={"origin": "https://example.invalid"})
    assert r.status_code == 403
    assert "result" not in r.json()
    assert "this computer's own window and agents only" in r.text


def test_a_request_from_the_apps_own_origin_is_answered(bare):
    r = bare.post("/api/mcp", json={"jsonrpc": "2.0", "id": 1, "method": "ping"}, headers={"origin": "http://127.0.0.1:8291"})
    assert well_formed(r.json(), 1)["result"] == {}


@pytest.mark.parametrize("method,params", [("tools/list", None), ("tools/call", {"name": "pattern_get", "arguments": {}})])
def test_tool_methods_refuse_with_the_build_instruction_when_the_engine_is_absent(bare, method, params):
    if not engine_absent():
        pytest.skip("the native engine is built here, so there is no refusal to observe")
    body = rpc(bare, method, params, req_id=11)
    assert -32099 <= body["error"]["code"] <= -32000
    assert "engine module is not built" in body["error"]["message"]
    assert "./formfabric engine build" in body["error"]["message"]


def test_tool_listing_follows_the_tool_shape_when_the_engine_is_built(bare):
    if engine_absent():
        pytest.skip("the native engine is not built on this machine; the refusal is tested instead")
    tools = rpc(bare, "tools/list")["result"]["tools"]
    assert tools
    names = [t["name"] for t in tools]
    assert len(set(names)) == len(names)
    for tool in tools:
        assert isinstance(tool["name"], str) and 1 <= len(tool["name"]) <= 128
        assert set(tool["name"]) <= set("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_-.")
        assert isinstance(tool["description"], str)
        assert tool["inputSchema"]["type"] == "object"


@pytest.mark.xfail(strict=True, reason="DEFECT: POST /api/mcp answers a message that has an id but no method with -32601 Method not found: None; JSON-RPC 2.0 section 5.1 says a message that is not a valid Request object gets -32600 Invalid Request")
def test_a_message_with_an_id_but_no_method_is_an_invalid_request(bare):
    r = bare.post("/api/mcp", json={"jsonrpc": "2.0", "id": 4})
    assert well_formed(r.json(), 4)["error"]["code"] == -32600


@pytest.mark.xfail(strict=True, reason="DEFECT: POST /api/mcp answers a request whose method is a number with -32601 Method not found; JSON-RPC 2.0 section 4 says method must be a String, so the reply should be -32600 Invalid Request")
def test_a_method_that_is_not_a_string_is_an_invalid_request(bare):
    r = bare.post("/api/mcp", json={"jsonrpc": "2.0", "id": 4, "method": 12})
    assert well_formed(r.json(), 4)["error"]["code"] == -32600


def handle(msg):
    from app import mcp_stdio

    out = mcp_stdio.handle(msg)
    return None if out is None else json.loads(out)


def test_stdio_ping_answers_an_empty_result_under_the_same_id():
    assert handle({"jsonrpc": "2.0", "id": "p-1", "method": "ping"}) == {"jsonrpc": "2.0", "id": "p-1", "result": {}}


def test_stdio_notification_gets_no_reply():
    assert handle({"jsonrpc": "2.0", "method": "notifications/initialized"}) is None


def test_stdio_unknown_method_gets_method_not_found():
    body = well_formed(handle({"jsonrpc": "2.0", "id": 2, "method": "prompts/sing"}), 2)
    assert body["error"]["code"] == -32601
    assert "prompts/sing" in body["error"]["message"]


@pytest.mark.parametrize("asked", ["2025-11-25", "2025-06-18", "2025-03-26", "2024-11-05"])
def test_stdio_initialize_echoes_each_published_protocol_revision(asked):
    result = handle({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": asked}})["result"]
    assert result["protocolVersion"] == asked
    assert result["serverInfo"] == {"name": "formfabric", "version": VERSION}


def test_stdio_initialize_answers_an_unknown_revision_with_its_latest():
    result = handle({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "1999-01-01"}})["result"]
    assert result["protocolVersion"] == "2025-11-25"


def test_stdio_and_http_give_the_same_instructions_and_server_name(bare):
    over_http = rpc(bare, "initialize", {"protocolVersion": "2025-11-25"})["result"]
    over_stdio = handle({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2025-11-25"}})["result"]
    assert over_http == over_stdio


@pytest.mark.parametrize("method", ["tools/list", "tools/call"])
def test_stdio_tool_methods_refuse_with_the_build_instruction_when_the_engine_is_absent(method):
    if not engine_absent():
        pytest.skip("the native engine is built here, so there is no refusal to observe")
    body = well_formed(handle({"jsonrpc": "2.0", "id": 8, "method": method, "params": {"name": "pattern_get"}}), 8)
    assert -32099 <= body["error"]["code"] <= -32000
    assert "./formfabric engine build" in body["error"]["message"]


def test_stdio_program_answers_one_line_per_request_and_none_for_notifications(tmp_path):
    lines = [
        "{broken",
        json.dumps([1, 2]),
        "",
        json.dumps({"jsonrpc": "2.0", "method": "notifications/initialized"}),
        json.dumps({"jsonrpc": "2.0", "id": 21, "method": "ping"}),
        json.dumps({"jsonrpc": "2.0", "id": "last", "method": "nothing/here"}),
    ]
    done = subprocess.run([sys.executable, "-m", "app.mcp_stdio"], input="\n".join(lines) + "\n", capture_output=True, text=True, encoding="utf-8", cwd=str(BACKEND), env=dict(os.environ), timeout=30)
    assert done.returncode == 0
    out = [json.loads(line) for line in done.stdout.splitlines()]
    assert [o["id"] for o in out] == [None, None, 21, "last"]
    assert [o.get("error", {}).get("code") for o in out] == [-32700, -32600, None, -32601]
    assert out[2]["result"] == {}
    for o in out:
        well_formed(o, o["id"])
    if engine_absent():
        assert "engine module is not built" in done.stderr


@pytest.mark.xfail(strict=True, reason="DEFECT: the stdio MCP server answers a message that has an id but no method with -32601 Method not found: None; JSON-RPC 2.0 section 5.1 says an invalid Request object gets -32600 Invalid Request")
def test_stdio_message_with_an_id_but_no_method_is_an_invalid_request():
    assert handle({"jsonrpc": "2.0", "id": 4})["error"]["code"] == -32600


@pytest.mark.xfail(strict=True, reason="DEFECT: the stdio MCP server answers params given as an array with -32603 Internal error (AttributeError); JSON-RPC 2.0 section 5.1 gives -32602 Invalid params for invalid method parameters, as the HTTP endpoint does")
def test_stdio_program_answers_params_of_the_wrong_kind_with_invalid_params():
    done = subprocess.run([sys.executable, "-m", "app.mcp_stdio"], input=json.dumps({"jsonrpc": "2.0", "id": 6, "method": "initialize", "params": [1]}) + "\n", capture_output=True, text=True, encoding="utf-8", cwd=str(BACKEND), env=dict(os.environ), timeout=30)
    assert json.loads(done.stdout.splitlines()[0])["error"]["code"] == -32602
