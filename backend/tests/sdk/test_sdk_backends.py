import json
import sys
import urllib.error

import pytest

from .support import TOKEN, CATALOGUE, FakeEngine, FakeOpener, clean_env, engine, engine_module, http_error, opener  # noqa: F401

import formfabric  # noqa: E402
from formfabric import backend as B  # noqa: E402
from formfabric.errors import BackendError, CommandError  # noqa: E402


def test_engine_backend_without_the_module_explains_how_to_build_it(monkeypatch):
    monkeypatch.setitem(sys.modules, "ffb_engine", None)
    with pytest.raises(BackendError, match="engine build"):
        formfabric.connect("engine")


def test_in_process_dispatch_round_trips_json_through_the_module(engine):
    engine.replies["sim.state"] = {"frame": 4}
    c = formfabric.connect("engine")
    assert c.name == "engine"
    assert c.call("sim.state") == {"frame": 4}
    assert c.catalogue()["commands"] == CATALOGUE["commands"]


def test_in_process_backend_reports_the_module_version(engine):
    assert B.InProcessBackend().version() == "0.0.0"


def test_auto_picks_the_engine_when_importable(engine, clean_env):
    assert formfabric.connect().name == "engine"


def test_auto_falls_back_to_http_when_the_module_is_missing(monkeypatch, clean_env):
    monkeypatch.setitem(sys.modules, "ffb_engine", None)
    assert formfabric.connect().name == "http"


def test_auto_picks_http_when_the_engine_url_variable_is_set(engine, monkeypatch, clean_env):
    monkeypatch.setenv("FFB_ENGINE_URL", "http://127.0.0.1:9999/")
    c = formfabric.connect()
    assert c.name == "http"
    assert c.backend.url == "http://127.0.0.1:9999"


def test_auto_picks_http_when_a_url_or_token_is_given(engine, clean_env):
    assert formfabric.connect(url="http://localhost:1").name == "http"
    assert formfabric.connect(token=TOKEN).name == "http"


def test_unknown_backend_name_is_refused(clean_env):
    with pytest.raises(BackendError, match="auto, engine or http, got cloud"):
        formfabric.connect("cloud")


def test_http_default_url_is_loopback_on_8280(clean_env):
    assert B.HttpBackend().url == "http://127.0.0.1:8280"


def test_http_url_uses_ffb_port_when_a_valid_number(monkeypatch, clean_env):
    monkeypatch.setenv("FFB_PORT", "9123")
    assert B.HttpBackend().url == "http://127.0.0.1:9123"


@pytest.mark.parametrize("port", ["abc", "0", "-5", ""])
def test_http_url_ignores_an_unusable_port_variable(monkeypatch, clean_env, port):
    monkeypatch.setenv("FFB_PORT", port)
    assert B.HttpBackend().url == "http://127.0.0.1:8280"


def test_http_explicit_url_wins_over_environment_and_loses_its_trailing_slash(monkeypatch, clean_env):
    monkeypatch.setenv("FFB_ENGINE_URL", "http://127.0.0.1:1")
    assert B.HttpBackend(url="http://localhost:7000//").url == "http://localhost:7000"


def test_dispatch_posts_name_and_args_as_json_to_the_dispatch_route(opener):
    opener.responder = lambda path, body: {"ok": True, "result": {"frame": 3}}
    out = formfabric.connect("http", check=False).call("sim.step", frames=3)
    assert out == {"frame": 3}
    req = opener.requests[0]
    assert req["method"] == "POST"
    assert req["url"] == "http://127.0.0.1:8280/api/engine/dispatch"
    assert req["body"] == {"name": "sim.step", "args": {"frames": 3}}
    assert req["headers"]["content-type"] == "application/json"


def test_catalogue_is_fetched_once_by_get_and_cached(opener):
    opener.responder = lambda path, body: CATALOGUE
    c = formfabric.connect("http")
    c.catalogue()
    c.catalogue()
    c.spec("sim.step")
    assert [(r["method"], r["path"]) for r in opener.requests] == [("GET", "/api/engine/catalogue")]


def test_checked_call_over_http_refuses_bad_arguments_without_posting(opener):
    opener.responder = lambda path, body: CATALOGUE
    c = formfabric.connect("http")
    with pytest.raises(formfabric.ArgumentError):
        c.call("sim.step", frames=0)
    assert all(r["method"] == "GET" for r in opener.requests)


def test_health_reads_the_health_route(opener):
    opener.responder = lambda path, body: {"status": "ok"}
    assert B.HttpBackend().health() == {"status": "ok"}
    assert opener.requests[0]["path"] == "/api/health"


def test_window_token_is_sent_in_the_window_header(opener):
    B.HttpBackend(token=TOKEN).dispatch("sim.state")
    assert opener.requests[0]["headers"]["x-ffb-window"] == TOKEN


def test_no_token_means_no_key_header_is_sent(opener):
    B.HttpBackend().dispatch("sim.state")
    assert "x-ffb-window" not in opener.requests[0]["headers"]
    assert "x-formfabric-inapp" not in opener.requests[0]["headers"]


def test_window_token_from_the_environment_is_used(opener, monkeypatch):
    monkeypatch.setenv("FFB_WINDOW_TOKEN", "  " + TOKEN + " ")
    B.HttpBackend().dispatch("sim.state")
    assert opener.requests[0]["headers"]["x-ffb-window"] == TOKEN


def test_explicit_token_wins_over_the_environment(opener, monkeypatch):
    monkeypatch.setenv("FFB_WINDOW_TOKEN", "e" * 40)
    B.HttpBackend(token="x" * 33).dispatch("sim.state")
    assert opener.requests[0]["headers"]["x-ffb-window"] == "x" * 33


def test_agent_token_from_the_environment_goes_in_the_inapp_header(opener, monkeypatch):
    monkeypatch.setenv("FFB_INAPP_TOKEN", "short")
    B.HttpBackend().dispatch("sim.state")
    assert opener.requests[0]["headers"]["x-formfabric-inapp"] == "short"
    assert "x-ffb-window" not in opener.requests[0]["headers"]


def test_window_token_wins_over_inapp_token_in_the_environment(opener, monkeypatch):
    monkeypatch.setenv("FFB_WINDOW_TOKEN", TOKEN)
    monkeypatch.setenv("FFB_INAPP_TOKEN", "other")
    B.HttpBackend().dispatch("sim.state")
    assert list(k for k in opener.requests[0]["headers"] if k.startswith("x-")) == ["x-ffb-window"]


def test_token_one_character_short_of_32_is_refused(clean_env):
    with pytest.raises(BackendError, match="at least 32 characters"):
        B.HttpBackend(token="a" * 31)
    assert B.HttpBackend(token="a" * 32).url


@pytest.mark.parametrize("url", ["http://example.com:8280", "https://127.0.0.1:8280", "http://127.0.0.1.evil.test", "http://[::1]:8280", "ftp://localhost"])
def test_token_is_never_sent_to_a_url_that_is_not_http_loopback(clean_env, url):
    with pytest.raises(BackendError, match="refusing to send the window token"):
        B.HttpBackend(url=url, token=TOKEN)


def test_agent_token_is_also_kept_from_remote_hosts(clean_env, monkeypatch):
    monkeypatch.setenv("FFB_INAPP_TOKEN", "abc")
    with pytest.raises(BackendError, match="refusing to send"):
        B.HttpBackend(url="http://example.com")


@pytest.mark.parametrize("url", ["http://127.0.0.1:1", "http://localhost:8280", "http://localhost"])
def test_token_may_go_to_http_loopback_hosts(clean_env, url):
    assert B.HttpBackend(url=url, token=TOKEN).url == url


def test_without_a_token_a_remote_url_is_allowed(clean_env):
    assert B.HttpBackend(url="http://example.com:80").url == "http://example.com:80"


def test_forbidden_reply_explains_the_window_key_and_the_in_process_backend(opener):
    opener.responder = lambda path, body: http_error(403, "bad key")
    with pytest.raises(BackendError) as err:
        B.HttpBackend(token=TOKEN).dispatch("sim.state")
    assert "bad key" in str(err.value)
    assert "in-process backend" in str(err.value)


def test_forbidden_reply_without_detail_says_forbidden(opener):
    opener.responder = lambda path, body: http_error(403)
    with pytest.raises(BackendError, match="forbidden"):
        B.HttpBackend().dispatch("sim.state")


def test_service_unavailable_reports_the_engine_not_built(opener):
    opener.responder = lambda path, body: http_error(503, "engine module not built")
    with pytest.raises(BackendError, match="engine module not built"):
        B.HttpBackend().dispatch("sim.state")


def test_other_http_errors_name_the_route_and_status(opener):
    opener.responder = lambda path, body: http_error(500, "kaput")
    with pytest.raises(BackendError, match="/api/engine/dispatch: HTTP 500 kaput"):
        B.HttpBackend().dispatch("sim.state")


def test_error_body_that_is_not_json_still_gives_a_backend_error(opener):
    opener.responder = lambda path, body: http_error(502)
    with pytest.raises(BackendError, match="HTTP 502"):
        B.HttpBackend().dispatch("sim.state")


def test_unreachable_service_names_the_url(opener):
    opener.responder = lambda path, body: urllib.error.URLError("connection refused")
    with pytest.raises(BackendError, match=r"cannot reach the formfabric service at http://127.0.0.1:8280: connection refused"):
        B.HttpBackend().dispatch("sim.state")


def test_reply_that_is_not_json_is_a_backend_error(opener):
    opener.responder = lambda path, body: b"<html>"
    with pytest.raises(BackendError, match="no usable reply"):
        B.HttpBackend().dispatch("sim.state")


def test_timeout_is_passed_to_the_transport(opener):
    B.HttpBackend(timeout=7.5).dispatch("sim.state")
    assert opener.requests[0]["timeout"] == 7.5


def test_engine_refusal_over_http_becomes_command_error(opener):
    opener.responder = lambda path, body: {"ok": False, "error": {"code": "no_body", "message": "load a body first"}}
    c = formfabric.connect("http", check=False)
    with pytest.raises(CommandError) as err:
        c.call("sim.step", frames=1)
    assert (err.value.code, err.value.message) == ("no_body", "load a body first")


def test_reply_without_error_details_gets_default_code_and_message():
    with pytest.raises(CommandError) as err:
        B.unwrap("x.y", {"ok": False})
    assert (err.value.code, err.value.message) == ("failed", "the command failed")


def test_ok_reply_without_result_returns_none():
    assert B.unwrap("x.y", {"ok": True}) is None


def test_client_as_context_manager_closes_its_backend(engine):
    closed = []
    c = formfabric.connect("engine")
    c.backend.close = lambda: closed.append(True)
    with c:
        pass
    assert closed == [True]


def test_unknown_command_name_is_refused_with_a_suggestion(engine):
    c = formfabric.connect("engine")
    with pytest.raises(formfabric.ArgumentError, match="did you mean") as err:
        c.call("sim.stpe", frames=1)
    assert "sim.step" in str(err.value)
    assert engine.calls == []


def test_unknown_command_with_no_near_match_gives_no_suggestion(engine):
    c = formfabric.connect("engine")
    with pytest.raises(formfabric.ArgumentError) as err:
        c.call("zzzzzzzz")
    assert "no such command" in str(err.value) and "did you mean" not in str(err.value)


def test_command_by_mcp_name_resolves_the_same_spec(engine):
    c = formfabric.connect("engine")
    assert c.spec("sim_step") is c.spec("sim.step")


def test_unchecked_client_sends_what_it_is_given(engine):
    c = formfabric.connect("engine", check=False)
    c.call("sim.step", frames=-4)
    assert engine.args_of("sim.step") == [{"frames": -4}]


def live_catalogue():
    extra = {
        "id": "cloth.tug",
        "name": "cloth_tug",
        "group": "cloth",
        "label": "Tug",
        "description": "Pull the hem.",
        "annotations": {},
        "inputSchema": {"type": "object", "additionalProperties": False, "properties": {"force": {"type": "number", "minimum": 0, "maximum": 5}, "from": {"type": "integer"}}, "required": ["force"]},
    }
    return {"commands": CATALOGUE["commands"] + [extra]}


def test_bind_gives_a_function_for_every_command_of_a_newer_engine(monkeypatch):
    fake = FakeEngine()
    monkeypatch.setitem(sys.modules, "ffb_engine", engine_module(fake, live_catalogue()))
    c = formfabric.connect("engine")
    bound = c.bind()
    assert len(bound) == len(CATALOGUE["commands"]) + 1
    bound.cloth_tug(force=2.5, from_=4)
    assert fake.args_of("cloth.tug") == [{"force": 2.5, "from": 4}]
    with pytest.raises(formfabric.ArgumentError):
        bound.cloth.tug(force=9)


def test_call_by_name_works_for_a_command_missing_from_the_snapshot(monkeypatch):
    fake = FakeEngine()
    monkeypatch.setitem(sys.modules, "ffb_engine", engine_module(fake, live_catalogue()))
    c = formfabric.connect("engine")
    c.call("cloth.tug", force=1)
    assert fake.names() == ["cloth.tug"]
    assert not hasattr(formfabric.commands, "cloth_tug")
