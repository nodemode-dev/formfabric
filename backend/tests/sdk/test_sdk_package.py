import json
import subprocess
import sys
import tomllib

import pytest

from .support import CATALOGUE, CATALOGUE_FILE, PYTHON_DIR, REPO

import formfabric  # noqa: E402
from formfabric import commands  # noqa: E402

PYPROJECT = tomllib.loads((PYTHON_DIR / "pyproject.toml").read_text(encoding="utf-8"))


def test_pyproject_names_the_package_formfabric_and_it_imports_under_that_name():
    assert PYPROJECT["project"]["name"] == "formfabric"
    assert formfabric.__name__ == "formfabric"


def test_pyproject_version_equals_the_version_the_package_reports():
    assert PYPROJECT["project"]["version"] == formfabric.__version__


def test_package_version_equals_the_apps_version_file():
    assert formfabric.__version__ == (REPO / "VERSION").read_text(encoding="utf-8").strip()


def test_pyproject_declares_no_dependencies_as_the_readme_promises():
    assert PYPROJECT["project"]["dependencies"] == []


def test_pyproject_ships_the_catalogue_snapshot_as_package_data():
    assert PYPROJECT["tool"]["setuptools"]["package-data"]["formfabric"] == ["catalogue.json"]
    assert PYPROJECT["tool"]["setuptools"]["packages"] == ["formfabric"]
    assert commands.SNAPSHOT == CATALOGUE_FILE and CATALOGUE_FILE.is_file()


def test_pyproject_readme_and_licence_are_the_ones_named_in_the_readme():
    assert (PYTHON_DIR / PYPROJECT["project"]["readme"]).is_file()
    assert PYPROJECT["project"]["license"]["text"] == "Apache-2.0"
    assert PYPROJECT["project"]["requires-python"] == ">=3.10"


def test_every_name_in_all_exists_and_the_readme_names_are_among_them():
    for name in formfabric.__all__:
        assert hasattr(formfabric, name), name
    for name in ("Garment", "Panel", "Link", "Sim", "sample_drape", "connect", "use", "ArgumentError", "CommandError", "BackendError", "commands", "gof"):
        assert name in formfabric.__all__


def test_error_classes_follow_the_readme_table():
    assert issubclass(formfabric.ArgumentError, formfabric.FormfabricError)
    assert issubclass(formfabric.CommandError, formfabric.FormfabricError)
    assert issubclass(formfabric.BackendError, formfabric.FormfabricError)
    assert not issubclass(formfabric.CommandError, formfabric.ArgumentError)


def test_importing_the_package_loads_only_the_standard_library_and_itself():
    code = (
        "import sys, json\n"
        "before = set(sys.modules)\n"
        f"sys.path.insert(0, {str(PYTHON_DIR)!r})\n"
        "import formfabric, formfabric.__main__\n"
        "new = {m.split('.')[0] for m in set(sys.modules) - before}\n"
        "print(json.dumps(sorted(m for m in new if m not in sys.stdlib_module_names and m != 'formfabric' and not m.startswith('_'))))\n"
    )
    done = subprocess.run([sys.executable, "-c", code], capture_output=True, text=True, timeout=30, cwd=str(PYTHON_DIR.parent))
    assert done.returncode == 0, done.stderr
    assert json.loads(done.stdout) == []


def test_snapshot_has_unique_ids_and_unique_mcp_names():
    ids = [s["id"] for s in CATALOGUE["commands"]]
    names = [s["name"] for s in CATALOGUE["commands"]]
    assert len(set(ids)) == len(ids) and len(set(names)) == len(names)


def test_snapshot_command_count_equals_the_functions_the_package_builds():
    assert len(commands.COMMANDS) == len(CATALOGUE["commands"])
    assert len(commands.load_snapshot()["commands"]) == len(commands.COMMANDS)


@pytest.mark.parametrize("command, args", [("sim.step", {"frames": 4}), ("pattern.rectangle", {"width": 400, "height": 300, "name": "swatch"}), ("sample.load", {"name": "dress", "force": True})])
def test_readme_example_calls_are_accepted_by_the_argument_check(command, args):
    from formfabric import schema

    schema.check(command, next(s for s in CATALOGUE["commands"] if s["id"] == command)["inputSchema"], args)


def test_function_name_replaces_dots_and_guards_keywords_and_digits():
    assert commands.function_name("sample.load") == "sample_load"
    assert commands.function_name("class") == "class_"
    assert commands.function_name("lambda") == "lambda_"
