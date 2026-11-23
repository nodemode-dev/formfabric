import os
import time

import pytest

REFUSAL = "Only the formfabric window can do this"


def run(client, cid, params=None):
    return client.post(f"/api/commands/{cid}", json=params or {})


def test_catalogue_lists_commands_in_id_order_and_counts_them(client):
    body = client.get("/api/commands").json()
    ids = [c["id"] for c in body["commands"]]
    assert ids == sorted(ids)
    assert len(set(ids)) == len(ids) == body["count"]
    assert {"terminal.open", "terminal.close", "terminal.list"} <= set(ids)


def test_group_filter_returns_exactly_the_commands_of_that_group(client):
    everything = client.get("/api/commands").json()["commands"]
    only = client.get("/api/commands", params={"group": "terminal"}).json()
    assert only["commands"] == [c for c in everything if c["id"].startswith("terminal.")]
    assert only["count"] == len(only["commands"]) > 0


def test_group_filter_with_an_unused_group_is_empty(client):
    assert client.get("/api/commands", params={"group": "no-such-group"}).json() == {"commands": [], "count": 0}


def test_describing_one_command_agrees_with_its_catalogue_entry(client):
    for entry in client.get("/api/commands").json()["commands"]:
        assert client.get(f"/api/commands/{entry['id']}").json() == entry


def test_describing_an_unknown_command_is_refused_by_name(client):
    r = client.get("/api/commands/pattern.fly")
    assert r.status_code == 404
    assert r.json() == {"ok": False, "error": "Unknown command: pattern.fly"}


def test_running_an_unknown_command_is_refused_by_name(client):
    r = run(client, "pattern.fly")
    assert r.status_code == 404
    assert r.json() == {"ok": False, "command": "pattern.fly", "error": "Unknown command: pattern.fly"}


def test_running_a_command_without_the_window_key_is_refused(bare):
    r = bare.post("/api/commands/terminal.list", json={})
    assert r.status_code == 403
    assert REFUSAL in r.text


def test_a_parameter_the_command_does_not_take_is_named_in_the_refusal(client):
    r = run(client, "terminal.list", {"zeta": 1, "alpha": 2})
    assert r.status_code == 400
    assert r.json()["error"] == "terminal.list does not take alpha, zeta"


@pytest.mark.parametrize("cid,params,message", [
    ("terminal.resize", {"id": "x", "cols": "80", "rows": 24}, "terminal.resize: cols must be an integer"),
    ("terminal.resize", {"id": "x", "cols": 80.5, "rows": 24}, "terminal.resize: cols must be an integer"),
    ("terminal.resize", {"id": "x", "cols": True, "rows": 24}, "terminal.resize: cols must be an integer"),
    ("terminal.read", {"id": 7}, "terminal.read: id must be a string"),
    ("agents.status", {"fresh": "yes"}, "agents.status: fresh must be a boolean"),
])
def test_a_wrongly_typed_parameter_is_refused_with_its_name_and_type(client, cid, params, message):
    r = run(client, cid, params)
    assert r.status_code == 400
    assert r.json() == {"ok": False, "command": cid, "error": message}


@pytest.mark.parametrize("cid,params", [
    ("terminal.read", {"id": "gone-1"}),
    ("terminal.write", {"id": "gone-1", "data": "x"}),
    ("terminal.resize", {"id": "gone-1", "cols": 80, "rows": 24}),
    ("terminal.restart", {"id": "gone-1"}),
])
def test_a_terminal_that_does_not_exist_is_refused_with_a_pointer_to_the_list(client, cid, params):
    r = run(client, cid, params)
    assert r.status_code == 404
    assert r.json()["error"] == "No terminal gone-1; terminal.list gives the ids"


def test_closing_a_terminal_that_is_already_gone_succeeds(client):
    r = run(client, "terminal.close", {"id": "gone-2"})
    assert r.status_code == 200
    assert r.json() == {"ok": True, "command": "terminal.close", "result": {"closed": "gone-2"}}


def test_opening_an_agent_program_of_an_unknown_kind_is_refused(client):
    before = run(client, "terminal.list").json()["result"]["count"]
    r = run(client, "terminal.agent", {"kind": "abacus"})
    assert r.status_code in (400, 404)
    assert run(client, "terminal.list").json()["result"]["count"] == before
    assert r.json()["ok"] is False
    assert "abacus" in r.json()["error"]


def test_opening_a_terminal_in_a_folder_that_does_not_exist_is_refused(client, tmp_path):
    before = run(client, "terminal.list").json()["result"]["count"]
    r = run(client, "terminal.open", {"cwd": str(tmp_path / "missing")})
    assert r.status_code in (400, 404)
    assert r.json()["ok"] is False
    assert run(client, "terminal.list").json()["result"]["count"] == before


@pytest.mark.skipif(os.name == "nt", reason="the test types POSIX shell arithmetic into the stand-in sh")
def test_text_typed_into_a_terminal_comes_back_from_reading_it(client, tmp_path):
    before = run(client, "terminal.list").json()["result"]["count"]
    opened = run(client, "terminal.open", {"cwd": str(tmp_path), "cols": 90, "rows": 20, "title": "sums"}).json()["result"]
    sid = opened["id"]
    try:
        listed = run(client, "terminal.list").json()["result"]
        assert listed["count"] == before + 1
        assert sid in [t["id"] for t in listed["terminals"]]
        typed = "echo $((123456 * 3))\r"
        assert run(client, "terminal.write", {"id": sid, "data": typed}).json()["result"] == {"written": len(typed)}
        text = ""
        for _ in range(40):
            text = run(client, "terminal.read", {"id": sid}).json()["result"]["text"]
            if "370368" in text:
                break
            time.sleep(0.1)
        assert "370368" in text
        assert "\x1b" not in text
        tail = run(client, "terminal.read", {"id": sid, "chars": 3}).json()["result"]["text"]
        assert len(tail) <= 3 and text.endswith(tail)
    finally:
        assert run(client, "terminal.close", {"id": sid}).json()["result"] == {"closed": sid}
    assert run(client, "terminal.list").json()["result"]["count"] == before
    assert run(client, "terminal.read", {"id": sid}).status_code == 404


@pytest.mark.skipif(os.name == "nt", reason="uses the POSIX stand-in sh")
def test_a_resized_terminal_reports_the_size_it_was_given(client, tmp_path):
    sid = run(client, "terminal.open", {"cwd": str(tmp_path)}).json()["result"]["id"]
    try:
        r = run(client, "terminal.resize", {"id": sid, "cols": 132, "rows": 43})
        assert r.json()["result"] == {"id": sid, "cols": 132, "rows": 43}
    finally:
        run(client, "terminal.close", {"id": sid})


def test_engine_routes_refuse_with_the_build_instruction_when_the_engine_is_absent(client):
    from app.services import engine

    if engine.available():
        pytest.skip("the native engine is built here, so there is no refusal to observe")
    for r in (client.get("/api/engine/catalogue"), client.get("/api/engine/stream"), client.post("/api/engine/dispatch", json={"name": "pattern.get", "args": {}})):
        assert r.status_code == 503
        assert "engine module is not built" in r.json()["detail"]
        assert "./formfabric engine build" in r.json()["detail"]


def test_engine_dispatch_without_the_window_key_is_refused(bare):
    r = bare.post("/api/engine/dispatch", json={"name": "pattern.get", "args": {}})
    assert r.status_code == 403
    assert REFUSAL in r.text


def test_engine_dispatch_with_an_overlong_name_is_rejected_as_invalid(client):
    r = client.post("/api/engine/dispatch", json={"name": "n" * 121, "args": {}})
    assert r.status_code == 422


def test_mcp_status_reports_no_tools_when_the_engine_is_absent(client):
    from app.services import engine

    if engine.available():
        pytest.skip("the native engine is built here")
    body = client.get("/api/mcp/status").json()
    assert body["endpoint"] == "/api/mcp"
    assert body["engine"] is False and body["tools"] == 0


def test_consent_answer_shows_in_status_and_reset_forgets_it(client):
    try:
        assert client.post("/api/mcp/consent", json={"allow": True}).json()["allowed"] is True
        declined = client.post("/api/mcp/consent", json={"allow": False}).json()
        assert declined["allowed"] is False and declined["declined"] is True
    finally:
        cleared = client.post("/api/mcp/reset").json()
    assert cleared["allowed"] is None and cleared["declined"] is False
    assert client.get("/api/mcp/status").json()["allowed"] is None


def test_consent_without_the_window_key_is_refused(bare, client):
    r = bare.post("/api/mcp/consent", json={"allow": True})
    assert r.status_code == 403
    assert client.get("/api/mcp/status").json()["allowed"] is None


def test_published_events_are_returned_in_order_with_rising_sequence_numbers(client):
    from app.services import events

    events.publish("lane.probe", step="first")
    events.publish("lane.probe", step="second")
    mine = [e for e in client.get("/api/events/recent").json()["events"] if e["type"] == "lane.probe"][-2:]
    assert [e["step"] for e in mine] == ["first", "second"]
    assert mine[1]["seq"] == mine[0]["seq"] + 1


def test_chat_image_is_served_back_byte_for_byte_with_its_type(client):
    from app.services import cmd_chat

    folder = cmd_chat.image_dir()
    folder.mkdir(parents=True, exist_ok=True)
    data = b"\x89PNG\r\n\x1a\n" + bytes(range(64))
    path = folder / "lane-commands-probe.png"
    note = folder / "lane-commands-probe.txt"
    path.write_bytes(data)
    note.write_text("x", encoding="utf-8")
    try:
        r = client.get("/api/chat/image/lane-commands-probe.png")
        assert r.status_code == 200
        assert r.content == data
        assert r.headers["content-type"] == "image/png"
        refused = client.get("/api/chat/image/lane-commands-probe.txt")
        assert refused.status_code == 404
        assert refused.json() == {"ok": False, "error": "No such chat image"}
    finally:
        path.unlink()
        note.unlink()
    assert client.get("/api/chat/image/lane-commands-probe.png").json() == {"ok": False, "error": "No such chat image"}
