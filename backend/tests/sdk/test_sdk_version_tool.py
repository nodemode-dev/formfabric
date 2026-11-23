import json
import shutil
import subprocess
import sys

import pytest

from .support import REPO

TOOL = REPO / "setup" / "version.py"
VERSION = "7.4.2"


def tree(root, version=VERSION):
    files = {
        "VERSION": f"{version}\n",
        "src-tauri/tauri.conf.json": json.dumps({"productName": "x", "version": version}),
        "src-tauri/Cargo.toml": f'[package]\nname = "formfabric"\nversion = "{version}"\n',
        "native/Cargo.toml": f'[workspace.package]\nversion = "{version}"\n',
        "backend/app/__init__.py": f'__version__ = "{version}"\n',
        "frontend/package.json": json.dumps({"name": "f", "version": version}),
        "frontend/src/version.ts": f"export const VERSION = '{version}'\n",
        "python/formfabric/__init__.py": f'__version__ = "{version}"\n',
        "python/pyproject.toml": f'[project]\nname = "formfabric"\nversion = "{version}"\n',
        "README.md": f"![v](https://img.shields.io/badge/version-v{version}-f97316)\n",
        "CHANGELOG.md": f"# Changelog\n\n## v{version} - 2026-01-01\n\nnotes\n",
        "native/Cargo.lock": f'[[package]]\nname = "ffb-core"\nversion = "{version}"\n\n[[package]]\nname = "serde"\nversion = "1.0.0"\n',
        "src-tauri/Cargo.lock": f'[[package]]\nname = "formfabric"\nversion = "{version}"\n',
    }
    for rel, text in files.items():
        path = root / rel
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8")
    (root / "setup").mkdir(exist_ok=True)
    shutil.copy(TOOL, root / "setup" / "version.py")
    return root


def run(root, *args):
    return subprocess.run([sys.executable, str(root / "setup" / "version.py"), *args], capture_output=True, text=True, timeout=30)


def replace(root, rel, old, new):
    path = root / rel
    text = path.read_text(encoding="utf-8")
    assert old in text
    path.write_text(text.replace(old, new), encoding="utf-8")


@pytest.fixture
def copy(tmp_path):
    return tree(tmp_path / "copy")


def test_consistent_copy_passes_and_names_the_version_and_site_count(copy):
    done = run(copy)
    assert done.returncode == 0
    assert f"one version, {VERSION}, across 10 sites and the changelog" in done.stdout


def test_print_gives_the_anchor_version_from_the_tauri_config(copy):
    done = run(copy, "print")
    assert (done.returncode, done.stdout.strip()) == (0, VERSION)


@pytest.mark.parametrize(
    "rel, old",
    [
        ("VERSION", VERSION),
        ("backend/app/__init__.py", VERSION),
        ("frontend/package.json", VERSION),
        ("frontend/src/version.ts", VERSION),
        ("python/formfabric/__init__.py", VERSION),
        ("python/pyproject.toml", VERSION),
        ("native/Cargo.toml", VERSION),
        ("src-tauri/Cargo.toml", VERSION),
        ("README.md", VERSION),
    ],
)
def test_a_planted_disagreement_in_one_site_is_reported_by_name_with_exit_one(copy, rel, old):
    replace(copy, rel, old, "7.4.3")
    done = run(copy)
    assert done.returncode == 1
    assert f"[formfabric] {rel}: 7.4.3 (expected {VERSION})" in done.stdout
    assert done.stdout.count("expected") == 1


def test_a_planted_disagreement_in_a_lock_file_is_reported_with_the_crate(copy):
    replace(copy, "native/Cargo.lock", f'name = "ffb-core"\nversion = "{VERSION}"', 'name = "ffb-core"\nversion = "0.0.1"')
    done = run(copy)
    assert done.returncode == 1
    assert f"native/Cargo.lock: ffb-core 0.0.1 (expected {VERSION})" in done.stdout


def test_a_lock_entry_of_an_unrelated_crate_is_ignored(copy):
    replace(copy, "native/Cargo.lock", 'name = "serde"\nversion = "1.0.0"', 'name = "serde"\nversion = "9.9.9"')
    assert run(copy).returncode == 0


def test_a_missing_site_is_reported_as_missing(copy):
    (copy / "frontend" / "src" / "version.ts").unlink()
    done = run(copy)
    assert done.returncode == 1
    assert f"frontend/src/version.ts: missing (expected {VERSION})" in done.stdout


def test_a_missing_lock_file_is_reported(copy):
    (copy / "src-tauri" / "Cargo.lock").unlink()
    done = run(copy)
    assert done.returncode == 1
    assert "src-tauri/Cargo.lock: missing" in done.stdout


def test_a_changelog_without_the_current_version_heading_fails(copy):
    replace(copy, "CHANGELOG.md", f"## v{VERSION}", "## v7.4.1")
    done = run(copy)
    assert done.returncode == 1
    assert f"CHANGELOG.md has no heading for {VERSION}" in done.stdout


def test_changelog_heading_may_omit_the_v(copy):
    replace(copy, "CHANGELOG.md", f"## v{VERSION}", f"## {VERSION}")
    assert run(copy).returncode == 0


def test_changelog_heading_for_a_longer_version_does_not_count(copy):
    replace(copy, "CHANGELOG.md", f"## v{VERSION} ", f"## v{VERSION}1 ")
    assert run(copy).returncode == 1


def test_several_disagreements_are_all_listed(copy):
    replace(copy, "VERSION", VERSION, "1.0.0")
    replace(copy, "python/pyproject.toml", VERSION, "2.0.0")
    done = run(copy)
    assert done.returncode == 1
    assert "VERSION: 1.0.0" in done.stdout and "python/pyproject.toml: 2.0.0" in done.stdout


def test_sites_lists_every_site_and_lock_with_its_version(copy):
    done = run(copy, "sites")
    assert done.returncode == 0
    rows = {line.split()[0]: line.split()[1:] for line in done.stdout.splitlines()}
    assert rows["VERSION"] == [VERSION]
    assert rows["python/pyproject.toml"] == [VERSION]
    assert rows["native/Cargo.lock"] == [VERSION]
    assert len(rows) == 12


def test_sites_shows_missing_for_an_absent_file(copy):
    (copy / "VERSION").unlink()
    done = run(copy, "sites")
    assert "MISSING" in done.stdout.splitlines()[0]


def test_unknown_command_prints_usage_and_exits_two(copy):
    done = run(copy, "bump")
    assert done.returncode == 2
    assert "usage" in done.stdout


def test_the_tool_only_reads_the_copy_it_sits_in(copy):
    before = {p: p.read_bytes() for p in copy.rglob("*") if p.is_file()}
    run(copy)
    run(copy, "sites")
    assert {p: p.read_bytes() for p in copy.rglob("*") if p.is_file()} == before


def test_the_real_repo_files_the_tool_reads_agree_with_each_other_on_the_python_package():
    done = subprocess.run([sys.executable, str(TOOL), "sites"], capture_output=True, text=True, timeout=30)
    rows = {line.split()[0]: line.split()[1] for line in done.stdout.splitlines() if len(line.split()) > 1}
    assert rows["python/formfabric/__init__.py"] == rows["python/pyproject.toml"] == rows["VERSION"]
