import pytest

DOUBLER = '''"""Double a number
Returns twice the number it is given."""
PARAMS = {
    "n": {"type": "integer", "label": "N", "default": 4, "min": 1, "max": 10},
    "mode": {"type": "choice", "label": "Mode", "default": "plain", "options": ["plain", "loud"]},
    "flag": {"type": "boolean", "label": "Flag", "default": False},
}


def run(ffb, args):
    print("mode is " + args["mode"])
    return {"twice": args["n"] * 2, "flag": args["flag"]}
'''

BROKEN = '''"""Always fails"""


def run(ffb, args):
    raise ValueError("boom 77")
'''

NEEDS_ENGINE = '''"""Asks the engine"""


def run(ffb, args):
    return ffb.call("pattern.get")
'''

NO_RUN = '''"""Not a plugin"""
X = 1
'''


@pytest.fixture
def plugin_dir(tmp_path, monkeypatch):
    (tmp_path / "lane_doubler.py").write_text(DOUBLER, encoding="utf-8")
    (tmp_path / "lane_broken.py").write_text(BROKEN, encoding="utf-8")
    (tmp_path / "lane_engine.py").write_text(NEEDS_ENGINE, encoding="utf-8")
    (tmp_path / "lane_norun.py").write_text(NO_RUN, encoding="utf-8")
    (tmp_path / "lane_syntax.py").write_text("def run(:\n", encoding="utf-8")
    monkeypatch.setenv("FFB_PLUGIN_DIR", str(tmp_path))
    return tmp_path


def listed(client):
    return {p["name"]: p for p in client.get("/api/plugins").json()["plugins"]}


def run(client, name, args=None):
    return client.post("/api/plugins/run", json={"name": name, "args": args or {}})


def test_a_plugin_file_is_listed_with_the_title_and_description_from_its_docstring(client, plugin_dir):
    found = listed(client)["lane_doubler"]
    assert found["title"] == "Double a number"
    assert found["description"] == "Returns twice the number it is given."
    assert found["params"]["n"] == {"type": "integer", "label": "N", "default": 4, "min": 1, "max": 10}
    assert found["params"]["mode"]["options"] == ["plain", "loud"]
    assert found["click"] is False


def test_files_without_a_run_function_or_that_do_not_parse_are_not_listed(client, plugin_dir):
    names = set(listed(client))
    assert {"lane_doubler", "lane_broken", "lane_engine"} <= names
    assert "lane_norun" not in names and "lane_syntax" not in names


def test_the_plugin_list_does_not_reveal_file_paths(client, plugin_dir):
    assert all("path" not in p for p in client.get("/api/plugins").json()["plugins"])


def test_plugins_are_listed_in_title_order(client, plugin_dir):
    titles = [p["title"].lower() for p in client.get("/api/plugins").json()["plugins"]]
    assert titles == sorted(titles)


def test_running_a_plugin_returns_its_result_and_what_it_printed(client, plugin_dir):
    body = run(client, "lane_doubler", {"n": 7, "mode": "loud", "flag": True}).json()
    assert body == {"ok": True, "result": {"twice": 14, "flag": True}, "output": "mode is loud\n"}


def test_running_a_plugin_with_no_arguments_uses_its_declared_defaults(client, plugin_dir):
    body = run(client, "lane_doubler").json()
    assert body == {"ok": True, "result": {"twice": 8, "flag": False}, "output": "mode is plain\n"}


def test_a_whole_number_sent_as_text_is_accepted_as_that_number(client, plugin_dir):
    assert run(client, "lane_doubler", {"n": "5"}).json()["result"]["twice"] == 10


@pytest.mark.parametrize("args,message", [
    ({"n": 0}, "n must be at least 1"),
    ({"n": 11}, "n must be at most 10"),
    ({"n": 2.5}, "n must be a whole number"),
    ({"n": "many"}, "n must be a number"),
    ({"n": True}, "n must be a number"),
    ({"mode": "quiet"}, "mode must be one of plain, loud"),
    ({"flag": "perhaps"}, "flag must be true or false"),
])
def test_an_argument_outside_what_the_plugin_declares_is_refused_before_it_runs(client, plugin_dir, args, message):
    body = run(client, "lane_doubler", args).json()
    assert body["ok"] is False
    assert body["error"]["code"] == "bad_args"
    assert body["error"]["message"] == message


def test_an_argument_the_plugin_does_not_declare_is_dropped(client, plugin_dir):
    body = run(client, "lane_doubler", {"n": 3, "stray": 99}).json()
    assert body["result"] == {"twice": 6, "flag": False}


def test_a_plugin_that_raises_is_reported_with_the_error_it_raised(client, plugin_dir):
    body = run(client, "lane_broken").json()
    assert body == {"ok": False, "error": {"code": "plugin_error", "message": "ValueError: boom 77"}}


def test_running_a_plugin_that_does_not_exist_is_refused_by_name(client, plugin_dir):
    r = run(client, "lane_absent")
    assert r.status_code == 404
    assert r.json()["detail"] == "no plugin lane_absent"


def test_running_a_plugin_without_the_window_key_is_refused(bare, plugin_dir):
    r = bare.post("/api/plugins/run", json={"name": "lane_doubler", "args": {}})
    assert r.status_code == 403
    assert "Only the formfabric window can do this" in r.text


def test_a_plugin_that_needs_the_engine_is_refused_with_the_build_instruction_when_it_is_absent(client, plugin_dir):
    from app.services import engine

    if engine.available():
        pytest.skip("the native engine is built here, so there is no refusal to observe")
    r = run(client, "lane_engine")
    assert r.status_code == 503
    assert "./formfabric engine build" in r.json()["detail"]
