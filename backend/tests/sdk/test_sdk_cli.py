import json
import os
import subprocess
import sys

import pytest

from .support import CATALOGUE, CATALOGUE_FILE, PYTHON_DIR, clean_env, engine, engine_module, FakeEngine, opener  # noqa: F401

import formfabric  # noqa: E402
from formfabric import __main__ as cli  # noqa: E402
from formfabric import commands  # noqa: E402

STUB = '''
import json, os

class Engine:
    def dispatch(self, name, raw):
        args = json.loads(raw)
        if name == "sim.step" and args.get("frames") == 13:
            return json.dumps({"ok": False, "error": {"code": "unlucky", "message": "thirteen is refused"}})
        return json.dumps({"ok": True, "result": {"name": name, "args": args}})

def catalogue():
    with open(os.environ["STUB_CATALOGUE"], encoding="utf-8") as handle:
        return handle.read()

def version():
    return "0.0.0"
'''


@pytest.fixture
def cli_env(tmp_path, clean_env):
    (tmp_path / "ffb_engine.py").write_text(STUB, encoding="utf-8")
    env = {k: v for k, v in os.environ.items() if not k.startswith("FFB_")}
    env["PYTHONPATH"] = os.pathsep.join([str(tmp_path), str(PYTHON_DIR)])
    env["STUB_CATALOGUE"] = str(CATALOGUE_FILE)
    return env


def run(env, *args):
    return subprocess.run([sys.executable, "-m", "formfabric", *args], capture_output=True, text=True, env=env, cwd=env["PYTHONPATH"].split(os.pathsep)[0], timeout=30)


def test_commands_lists_one_line_per_catalogue_command(cli_env):
    done = run(cli_env, "--backend", "engine", "commands")
    assert done.returncode == 0
    lines = done.stdout.splitlines()
    assert len(lines) == len(CATALOGUE["commands"])
    first = CATALOGUE["commands"][0]
    assert lines[0].split()[0] == first["id"] and lines[0].split()[1] == first["group"]
    assert lines[0].rstrip().endswith(first["label"])


def test_call_prints_the_result_as_json_with_values_parsed_as_json(cli_env):
    done = run(cli_env, "--backend", "engine", "call", "sim.step", "frames=7")
    assert done.returncode == 0
    assert json.loads(done.stdout) == {"name": "sim.step", "args": {"frames": 7}}


def test_call_values_that_are_not_json_are_passed_as_text(cli_env):
    done = run(cli_env, "--backend", "engine", "call", "sample.load", "name=tshirt", "force=true")
    assert json.loads(done.stdout)["args"] == {"name": "tshirt", "force": True}


def test_call_with_an_out_of_range_value_fails_with_exit_one_and_a_message_on_stderr(cli_env):
    done = run(cli_env, "--backend", "engine", "call", "sim.step", "frames=0")
    assert done.returncode == 1
    assert done.stdout == ""
    assert done.stderr.startswith("formfabric: sim.step: frames must be at least 1")


def test_call_with_an_unknown_command_suggests_the_near_one(cli_env):
    done = run(cli_env, "--backend", "engine", "call", "sim.stpe")
    assert done.returncode == 1
    assert "did you mean" in done.stderr and "sim.step" in done.stderr


def test_call_shows_the_engines_own_refusal_code_and_message(cli_env):
    done = run(cli_env, "--backend", "engine", "call", "sim.step", "frames=13")
    assert done.returncode == 1
    assert "thirteen is refused (unlucky)" in done.stderr


def test_engine_backend_without_the_module_exits_one_with_the_build_hint(tmp_path, clean_env):
    (tmp_path / "ffb_engine.py").write_text("raise ImportError('absent')\n", encoding="utf-8")
    env = {k: v for k, v in os.environ.items() if not k.startswith("FFB_")}
    env["PYTHONPATH"] = os.pathsep.join([str(tmp_path), str(PYTHON_DIR)])
    done = run(env, "--backend", "engine", "commands")
    assert done.returncode == 1
    assert "engine build" in done.stderr


def test_http_backend_with_nothing_listening_exits_one_naming_the_url(cli_env):
    done = run(cli_env, "--backend", "http", "--url", "http://127.0.0.1:1", "commands")
    assert done.returncode == 1
    assert "cannot reach the formfabric service at http://127.0.0.1:1" in done.stderr


def test_missing_subcommand_is_a_usage_error(cli_env):
    done = run(cli_env, "--backend", "engine")
    assert done.returncode == 2
    assert "usage:" in done.stderr


def test_unknown_backend_choice_is_a_usage_error(cli_env):
    done = run(cli_env, "--backend", "cloud", "commands")
    assert done.returncode == 2


def test_snapshot_print_writes_the_connected_catalogue_to_stdout_only(cli_env):
    before = CATALOGUE_FILE.read_bytes()
    done = run(cli_env, "--backend", "engine", "snapshot", "--print")
    assert done.returncode == 0
    assert json.loads(done.stdout) == CATALOGUE
    assert CATALOGUE_FILE.read_bytes() == before


def test_snapshot_write_then_load_round_trips_the_catalogue(engine, monkeypatch, tmp_path, capsys):
    target = tmp_path / "catalogue.json"
    monkeypatch.setattr(commands, "SNAPSHOT", target)
    assert cli.main(["--backend", "engine", "snapshot"]) == 0
    assert capsys.readouterr().out.strip() == f"{target} {len(CATALOGUE['commands'])} commands"
    assert commands.load_snapshot() == CATALOGUE


def test_load_snapshot_of_a_missing_file_is_an_empty_command_list(monkeypatch, tmp_path):
    monkeypatch.setattr(commands, "SNAPSHOT", tmp_path / "nothing.json")
    assert commands.load_snapshot() == {"commands": []}


def test_main_in_process_prints_the_commands_through_a_http_stub(opener, capsys):
    opener.responder = lambda path, body: CATALOGUE
    assert cli.main(["--backend", "http", "commands"]) == 0
    assert len(capsys.readouterr().out.splitlines()) == len(CATALOGUE["commands"])
