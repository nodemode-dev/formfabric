import re

import pytest

from conftest import window_headers

READS = {"GET", "HEAD", "OPTIONS"}
OPEN = {("POST", "/api/mcp"), ("POST", "/api/window/key")}
REFUSED = {"detail": "Only the formfabric window can do this"}
WRONG = {"x-ffb-window": "x" * 43}
PRIVATE_READS = ["/api/recent", "/api/mcp/window", "/api/events/recent", "/api/recovery", "/api/garments"]


def write_operations(app):
    return [(m.upper(), path) for path, item in app.openapi()["paths"].items() for m in item if m.upper() not in READS]


def inapp_headers() -> dict:
    from app.services import agents

    return {agents.INAPP_HEADER: agents.INAPP_TOKEN}


def test_the_open_list_is_short_and_named():
    from app.core import window_guard

    assert set(window_guard.OPEN) == {"/api/mcp", "/api/window/key"}


def test_every_write_route_needs_the_window_key(app, bare):
    ops = write_operations(app)
    assert len(ops) > 15
    answered = []
    for method, path in ops:
        if (method, path) in OPEN:
            continue
        url = re.sub(r"\{[^}]+\}", "x", path)
        for headers in ({}, WRONG, inapp_headers()):
            r = bare.request(method, url, json={}, headers=headers)
            if r.status_code != 403 or r.json() != REFUSED:
                answered.append(f"{method} {path} {r.status_code}")
    assert not answered, "write routes that answer without the window key: " + ", ".join(answered)


def test_the_window_key_opens_a_write_route(client):
    r = client.post("/api/mcp/window", json={})
    assert r.status_code != 403


@pytest.mark.parametrize("path", PRIVATE_READS)
def test_private_reads_refuse_without_the_key(app, bare, path):
    assert path in app.openapi()["paths"]
    for headers in ({}, WRONG, inapp_headers()):
        r = bare.get(path, headers=headers)
        assert r.status_code == 403 and r.json() == REFUSED


@pytest.mark.parametrize("path", PRIVATE_READS)
def test_private_reads_answer_the_window(app, client, path):
    assert path in app.openapi()["paths"]
    assert client.get(path).status_code == 200


def test_the_agent_token_opens_the_mcp_route_only(bare):
    r = bare.post("/api/mcp", json={"jsonrpc": "2.0", "id": 1, "method": "ping", "params": {}}, headers=inapp_headers())
    assert r.status_code == 200 and r.json()["result"] == {}
    r = bare.post("/api/mcp/consent", json={"allow": True}, headers=inapp_headers())
    assert r.status_code == 403 and r.json() == REFUSED
    r = bare.post("/api/commands/agents.prefs", json={"claudeBin": "/bin/true"}, headers=inapp_headers())
    assert r.status_code == 403 and r.json() == REFUSED


def test_the_window_key_is_given_once(bare):
    from app.core import config, window_guard

    first = bare.post("/api/window/key")
    assert first.status_code == 200 and first.json()["key"] == config.window_token()
    again = bare.post("/api/window/key")
    assert again.status_code == 409
    assert window_guard.CLAIM["secondClaim"] is True
    assert bare.get("/api/window/key").json() == {"secondClaim": True}


def test_the_shell_given_key_closes_the_claim(monkeypatch, bare):
    from app.core import config

    monkeypatch.setattr(config, "_TOKEN", "k" * 64)
    assert bare.post("/api/window/key").status_code == 404
    assert bare.post("/api/mcp/window", json={}, headers={"x-ffb-window": "k" * 64}).status_code != 403
    assert window_headers()["x-ffb-window"] == "k" * 64


def test_a_wrong_host_is_refused_even_with_the_key(bare):
    r = bare.post("/api/mcp/window", json={}, headers={**window_headers(), "host": "evil.example:8291"})
    assert r.status_code in (400, 403)
