import pytest
from starlette.websockets import WebSocketDisconnect

from conftest import ORIGIN, window_headers

from app.core import config, window_guard

from .support import LOCAL_REFUSED, OPEN, REFUSED, concrete, data_folder, write_operations

OWN_HOST = ORIGIN.split("//")[1]
FRONT = f"http://localhost:{config.FRONTEND_PORT}"
BAD_ORIGINS = ["http://evil.example", "https://127.0.0.1:8291", "null", "http://localhost:5280.evil.example", "http://localhost:5280@evil.example", "http://localhost:notaport", "http://127.0.0.1:7777", "file://", "ftp://localhost:5280"]
BAD_HOSTS = ["evil.example", "evil.example:8291", "localhost.evil.example:8291", "127.0.0.1.nip.example:8291", "10.0.0.5:8291", "0.0.0.0:8291"]


def keyed(**extra):
    return {**window_headers(), **extra}


@pytest.mark.parametrize("origin", BAD_ORIGINS)
def test_every_write_route_refuses_a_foreign_origin_even_with_the_window_key(app, bare, origin):
    answered = []
    for method, path in write_operations(app):
        r = bare.request(method, concrete(path), json={}, headers=keyed(origin=origin))
        if r.status_code != 403 or r.json() != LOCAL_REFUSED:
            answered.append(f"{method} {path} {r.status_code}")
    assert not answered


@pytest.mark.parametrize("host", BAD_HOSTS)
def test_every_write_route_refuses_a_foreign_host_even_with_the_window_key(app, bare, host):
    answered = []
    for method, path in write_operations(app):
        r = bare.request(method, concrete(path), json={}, headers=keyed(host=host))
        if r.status_code != 403 or r.json() != LOCAL_REFUSED:
            answered.append(f"{method} {path} {r.status_code}")
    assert not answered


def test_every_write_route_refuses_a_cross_site_request_with_no_origin(app, bare):
    answered = []
    for method, path in write_operations(app):
        r = bare.request(method, concrete(path), json={}, headers=keyed(**{"sec-fetch-site": "cross-site"}))
        if r.status_code != 403 or r.json() != LOCAL_REFUSED:
            answered.append(f"{method} {path} {r.status_code}")
    assert not answered


def test_every_write_route_refuses_a_loopback_host_the_window_does_not_use_even_with_the_key(app, bare):
    answered = []
    for method, path in write_operations(app):
        if path in OPEN:
            continue
        for host in ("localhost:9999", "127.0.0.1", "[::1]:8291"):
            r = bare.request(method, concrete(path), json={}, headers=keyed(host=host))
            if r.status_code != 403 or r.json() != REFUSED:
                answered.append(f"{method} {path} {host} {r.status_code}")
    assert not answered


def test_a_write_to_a_path_the_app_does_not_have_is_refused_for_want_of_the_key_before_it_is_found_missing(bare, client):
    assert bare.post("/api/no-such-route", json={}).status_code == 403
    assert bare.delete("/api/no-such-route").json() == REFUSED
    assert client.post("/api/no-such-route", json={}).status_code == 404


def test_a_write_with_a_method_the_route_lacks_is_refused_without_the_key_and_405_with_it(bare, client):
    assert bare.put("/api/health", json={}).status_code == 403
    assert client.put("/api/health", json={}).status_code == 405


@pytest.mark.parametrize("key", ["", " ", "x", "X" * 43])
def test_a_wrong_window_key_is_refused(bare, key):
    r = bare.post("/api/commands/agents.prefs", json={}, headers={"x-ffb-window": key})
    assert r.status_code == 403 and r.json() == REFUSED


def test_the_right_key_with_a_letter_changed_or_padding_is_refused(bare):
    real = config.window_token()
    flipped = ("A" if real[0] != "A" else "B") + real[1:]
    for key in (flipped, real + " ", real[:-1], real + real):
        r = bare.post("/api/commands/agents.prefs", json={}, headers={"x-ffb-window": key})
        assert r.status_code == 403, key


def test_the_right_key_from_the_window_host_gets_past_both_guards(client, bare, data_folder):
    r = client.post("/api/commands/agents.prefs", json={})
    assert r.status_code == 200
    r = bare.post("/api/commands/agents.prefs", json={}, headers=keyed(host=f"localhost:{config.engine_port()}"))
    assert r.status_code == 200


@pytest.mark.parametrize("origin", [FRONT, f"http://127.0.0.1:{config.FRONTEND_PORT}", ORIGIN, f"http://localhost:{config.engine_port()}", "tauri://localhost", "http://tauri.localhost", FRONT.upper()])
def test_a_read_from_each_window_origin_is_answered(bare, origin):
    r = bare.get("/api/health", headers={"origin": origin})
    assert r.status_code == 200 and r.json()["ok"] is True


@pytest.mark.parametrize("origin", BAD_ORIGINS)
def test_a_read_from_a_foreign_origin_is_refused_with_the_stated_message(bare, origin):
    r = bare.get("/api/health", headers={"origin": origin})
    assert r.status_code == 403 and r.json() == LOCAL_REFUSED


@pytest.mark.parametrize("host", BAD_HOSTS)
def test_a_read_sent_to_a_foreign_host_is_refused_even_from_an_allowed_origin(bare, host):
    r = bare.get("/api/health", headers={"host": host, "origin": ORIGIN})
    assert r.status_code == 403 and r.json() == LOCAL_REFUSED


@pytest.mark.parametrize("host", ["localhost:8291", "LOCALHOST:8291", "127.0.0.1:8291", "[::1]:8291", "localhost", "127.0.0.1"])
def test_a_read_sent_to_any_loopback_name_is_answered(bare, host):
    assert bare.get("/api/health", headers={"host": host}).status_code == 200


@pytest.mark.parametrize("site,status", [("same-origin", 200), ("same-site", 200), ("none", 200), ("cross-site", 403), ("Cross-Site", 403)])
def test_a_request_with_no_origin_is_judged_by_where_the_browser_says_it_came_from(bare, site, status):
    assert bare.get("/api/health", headers={"sec-fetch-site": site}).status_code == status


def test_a_request_from_a_program_with_no_browser_headers_is_answered(bare):
    assert bare.get("/api/health").json()["app"] == "formfabric"


def test_a_page_served_from_another_local_port_is_answered_only_when_it_is_the_host_it_asked(bare):
    ok = bare.get("/api/health", headers={"host": "127.0.0.1:7777", "origin": "http://127.0.0.1:7777"})
    assert ok.status_code == 200
    no = bare.get("/api/health", headers={"host": "127.0.0.1:7777", "origin": "http://127.0.0.1:7778"})
    assert no.status_code == 403


def test_a_socket_to_a_foreign_host_is_closed_with_4403_before_it_opens(client):
    with pytest.raises(WebSocketDisconnect) as caught:
        with client.websocket_connect("/api/terminal/none", headers={"host": "evil.example:8291"}):
            pass
    assert caught.value.code == 4403


def test_a_socket_from_a_foreign_origin_is_closed_with_4403(client):
    with pytest.raises(WebSocketDisconnect) as caught:
        with client.websocket_connect("/api/terminal/none", headers={"host": OWN_HOST, "origin": "http://evil.example"}):
            pass
    assert caught.value.code == 4403


def test_a_socket_from_the_window_host_with_no_key_is_closed_with_4403_for_want_of_it(bare):
    with pytest.raises(WebSocketDisconnect) as caught:
        with bare.websocket_connect("/api/terminal/none", headers={"host": OWN_HOST}) as ws:
            ws.send_text("")
            ws.receive_text()
    assert caught.value.code == 4403


def test_a_socket_that_sends_the_wrong_key_is_closed_with_4403(bare):
    with pytest.raises(WebSocketDisconnect) as caught:
        with bare.websocket_connect("/api/terminal/none", headers={"host": OWN_HOST}) as ws:
            ws.send_text("wrong")
            ws.receive_text()
    assert caught.value.code == 4403


def test_a_socket_with_the_key_in_its_header_reaches_the_session_lookup(client):
    with pytest.raises(WebSocketDisconnect) as caught:
        with client.websocket_connect("/api/terminal/none", headers={"host": OWN_HOST}) as ws:
            ws.receive_text()
    assert caught.value.code == 4404


def test_a_socket_that_sends_the_key_as_its_first_message_reaches_the_session_lookup(bare):
    with pytest.raises(WebSocketDisconnect) as caught:
        with bare.websocket_connect("/api/terminal/none", headers={"host": OWN_HOST}) as ws:
            ws.send_text(config.window_token())
            ws.receive_text()
    assert caught.value.code == 4404


def test_reads_do_not_need_the_key_unless_the_route_is_private(bare):
    assert bare.get("/api/health").status_code == 200
    assert bare.get("/api/capabilities").status_code == 200
    assert bare.get("/api/recent").status_code == 403


def test_a_preflight_from_the_dev_page_is_answered_without_the_key_and_names_that_origin(bare):
    r = bare.options("/api/engine/dispatch", headers={"origin": FRONT, "access-control-request-method": "POST"})
    assert r.status_code == 200
    assert r.headers["access-control-allow-origin"] == FRONT
    assert "POST" in r.headers["access-control-allow-methods"]


def test_a_preflight_from_an_origin_outside_the_cors_list_is_turned_down(bare):
    r = bare.options("/api/engine/dispatch", headers={"origin": ORIGIN, "access-control-request-method": "POST"})
    assert r.status_code == 400 and r.text == "Disallowed CORS origin"


def test_a_refused_write_from_the_dev_page_still_carries_the_cors_header_so_the_page_can_read_why(bare):
    r = bare.post("/api/engine/dispatch", json={}, headers={"origin": FRONT})
    assert r.status_code == 403 and r.json() == REFUSED
    assert r.headers["access-control-allow-origin"] == FRONT


def test_the_window_key_exact_origin_form_drops_default_ports_and_case():
    assert window_guard.exact("HTTP://Localhost:80") == "http://localhost"
    assert window_guard.exact("https://example.test:443") == "https://example.test"
    assert window_guard.exact("http://127.0.0.1:8291") == "http://127.0.0.1:8291"
    assert window_guard.exact("tauri://localhost") == "tauri://localhost"


@pytest.mark.parametrize("text", ["localhost", "http://a/b", "http://", "://x", "http://a b", ""])
def test_a_malformed_origin_has_no_exact_form(text):
    assert window_guard.exact(text) is None


def test_the_hosts_the_window_may_use_are_the_loopback_names_with_the_engine_port():
    hosts = window_guard.allowed_hosts()
    assert {f"127.0.0.1:{config.engine_port()}", f"localhost:{config.engine_port()}"} <= hosts
    assert all(h.startswith(("127.0.0.1", "localhost")) for h in hosts)


def test_a_request_is_from_the_window_only_with_both_the_host_and_the_key():
    host = f"127.0.0.1:{config.engine_port()}"
    key = config.window_token()
    assert window_guard.from_window({"host": host, "x-ffb-window": key}) is True
    assert window_guard.from_window({"host": host}) is False
    assert window_guard.from_window({"x-ffb-window": key}) is False
    assert window_guard.from_window({"host": "evil.example", "x-ffb-window": key}) is False
    assert window_guard.from_window({"host": host, "x-ffb-window": key[::-1]}) is False
