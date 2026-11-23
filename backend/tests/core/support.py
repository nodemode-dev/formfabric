import re
import subprocess
import sys

import pytest

from conftest import BACKEND

READS = {"GET", "HEAD", "OPTIONS"}
OPEN = {"/api/mcp", "/api/window/key"}
REFUSED = {"detail": "Only the formfabric window can do this"}
LOCAL_REFUSED = {"detail": "formfabric answers this computer's own window and agents only"}
PNG = b"\x89PNG\r\n\x1a\n" + b"\x00" * 32


def write_operations(app):
    return [(m.upper(), p) for p, item in app.openapi()["paths"].items() for m in item if m.upper() not in READS]


def concrete(path):
    return re.sub(r"\{[^}]+\}", "x", path)


def run_python(code, **env):
    import os

    clean = {k: v for k, v in os.environ.items() if not k.startswith(("FFB_", "FORMFABRIC_"))}
    clean.update(env)
    done = subprocess.run([sys.executable, "-c", code], cwd=BACKEND, capture_output=True, text=True, timeout=60, env=clean)
    assert done.returncode == 0, done.stderr
    return done.stdout.split()


@pytest.fixture
def data_folder(tmp_path, monkeypatch):
    folder = tmp_path / "data"
    monkeypatch.setenv("FORMFABRIC_DATA_DIR", str(folder))
    return folder


@pytest.fixture
def no_keychain():
    import keyring
    from keyring.backends import fail

    from app.core import seal

    before = keyring.get_keyring()
    keyring.set_keyring(fail.Keyring())
    seal.reset()
    yield
    keyring.set_keyring(before)
    seal.reset()
