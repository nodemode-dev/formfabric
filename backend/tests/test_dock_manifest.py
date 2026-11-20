import hashlib
import importlib.util
import json
import shutil

from conftest import REPO

DOCK = REPO / "frontend" / "src" / "dock"


def copied_agents(tmp_path):
    copy = tmp_path / "repo"
    (copy / "frontend" / "src").mkdir(parents=True)
    shutil.copytree(DOCK, copy / "frontend" / "src" / "dock")
    shutil.copytree(REPO / "setup", copy / "setup", ignore=shutil.ignore_patterns("__pycache__"))
    spec = importlib.util.spec_from_file_location("copied_agents", copy / "setup" / "agents.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return copy, module


def test_the_shared_dock_files_match_their_manifest():
    kept = json.loads((DOCK / "MANIFEST.json").read_text(encoding="utf-8"))
    assert kept["module"] == "dock" and kept["version"] == "2.2.1"
    for name, digest in kept["files"].items():
        assert hashlib.sha256((DOCK / name).read_bytes()).hexdigest() == digest, name


def test_the_engine_reports_the_panel_version_it_was_built_for():
    from app.services import agent_programs

    assert agent_programs.DOCK_CONTRACT == "2.2.1"


def test_the_agents_check_finds_a_changed_dock_file(tmp_path):
    copy, agents = copied_agents(tmp_path)
    assert agents.dock_problems() == []
    target = copy / "frontend" / "src" / "dock" / "strings.ts"
    target.write_text(target.read_text(encoding="utf-8") + "\n", encoding="utf-8")
    assert agents.dock_problems() == ["frontend/src/dock/strings.ts: does not match MANIFEST.json (the shared dock module changes only as a whole new version)"]


def test_the_agents_check_finds_an_older_panel(tmp_path):
    copy, agents = copied_agents(tmp_path)
    manifest = copy / "frontend" / "src" / "dock" / "MANIFEST.json"
    held = json.loads(manifest.read_text(encoding="utf-8"))
    held["version"] = "2.2.0"
    manifest.write_text(json.dumps(held), encoding="utf-8")
    assert agents.dock_problems() == ["frontend/src/dock/MANIFEST.json: version 2.2.0, this app expects 2.2.1"]


def test_only_stand_in_agent_programs_are_found(tmp_path):
    import os

    from app.services import agent_lookup

    assert agent_lookup.system_folders() == []
    for name in ("claude", "codex", "cursor-agent", "agy"):
        found = shutil.which(name)
        assert found is None or "formfabric-tests-" in found, found
    assert os.environ["SHELL"].endswith(os.path.join("shell", "sh")) or os.name == "nt"
