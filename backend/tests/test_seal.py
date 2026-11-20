import asyncio
import json

import pytest

WRITTEN_BY_0_26_0 = '{\n "claudeFolder": "/tmp/somewhere",\n "claudeBin": "/opt/stand-in/claude",\n "claudeFirstMessage": "",\n "agentsAllowed": false\n}'
ALLOWED_BY_0_26_0 = '{\n "claudeFolder": "",\n "claudeBin": "/opt/stand-in/claude",\n "claudeFirstMessage": "",\n "agentsAllowed": true\n}'


class StandInKeychain:
    priority = 1

    def __init__(self):
        self.items = {}

    def get_password(self, service, account):
        return self.items.get((service, account))

    def set_password(self, service, account, password):
        self.items[(service, account)] = password

    def delete_password(self, service, account):
        self.items.pop((service, account), None)


@pytest.fixture
def prefs_file():
    from app.core import config

    path = config.data_dir() / "preferences.json"
    path.unlink(missing_ok=True)
    yield path
    path.unlink(missing_ok=True)


@pytest.fixture
def keychain(prefs_file):
    import keyring
    from keyring.backend import KeyringBackend

    from app.core import seal

    class Backend(StandInKeychain, KeyringBackend):
        def __init__(self):
            KeyringBackend.__init__(self)
            StandInKeychain.__init__(self)

    before = keyring.get_keyring()
    backend = Backend()
    keyring.set_keyring(backend)
    seal.reset()
    yield backend
    keyring.set_keyring(before)
    seal.reset()


def edit(path, **changes):
    data = json.loads(path.read_text())
    data.update(changes)
    path.write_text(json.dumps(data, indent=1))


def test_a_choice_written_by_0_26_0_is_honoured_after_the_update(keychain, prefs_file):
    from app.core import prefs, seal

    prefs_file.write_text(WRITTEN_BY_0_26_0)
    first = prefs.load()
    assert first["agentsAllowed"] is False
    assert first["claudeBin"] == "/opt/stand-in/claude"
    assert first["claudeFolder"] == "/tmp/somewhere"
    assert (first["seal"], first["sealBroken"]) == ("ok", False)
    assert isinstance(json.loads(prefs_file.read_text()).get("seal"), str)
    seal.reset()
    again = prefs.load()
    assert (again["agentsAllowed"], again["claudeBin"], again["seal"]) == (False, "/opt/stand-in/claude", "ok")


def test_an_allowed_choice_written_by_0_26_0_keeps_its_program_path(keychain, prefs_file):
    from app.core import prefs

    prefs_file.write_text(ALLOWED_BY_0_26_0)
    got = prefs.load()
    assert (got["agentsAllowed"], got["claudeBin"], got["seal"], got["sealBroken"]) == (True, "/opt/stand-in/claude", "ok", False)


def test_a_program_path_changed_outside_the_app_is_not_used(keychain, prefs_file):
    from app.core import prefs

    prefs.update({"agentsAllowed": False})
    edit(prefs_file, claudeBin="/tmp/elsewhere/claude")
    got = prefs.load()
    assert (got["claudeBin"], got["agentsAllowed"], got["seal"], got["sealBroken"]) == ("", False, "broken", True)


def test_a_seal_that_does_not_match_reads_as_ask(keychain, prefs_file):
    from app.core import prefs

    prefs.update({"claudeBin": "/opt/stand-in/claude", "agentsAllowed": True})
    edit(prefs_file, seal="0" * 64)
    got = prefs.load()
    assert (got["agentsAllowed"], got["claudeBin"], got["sealBroken"]) == (True, "", True)


def test_a_file_with_no_seal_after_the_first_launch_is_not_adopted(keychain, prefs_file):
    from app.core import prefs

    prefs.load()
    prefs_file.write_text(ALLOWED_BY_0_26_0)
    got = prefs.load()
    assert (got["claudeBin"], got["seal"], got["sealBroken"]) == ("", "broken", True)


def test_the_window_sets_a_sealed_choice(keychain, prefs_file):
    from app.core import prefs

    prefs.update({"claudeBin": "/opt/stand-in/claude"})
    got = prefs.load()
    assert (got["claudeBin"], got["seal"]) == ("/opt/stand-in/claude", "ok")
    edit(prefs_file, claudeFolder="/tmp/other")
    assert prefs.load()["seal"] == "ok"


def test_a_broken_path_is_not_sealed_by_a_later_write(keychain, prefs_file):
    from app.core import prefs

    prefs.update({"agentsAllowed": True})
    edit(prefs_file, claudeBin="/tmp/elsewhere/claude")
    after = prefs.update({"claudeFolder": "/tmp/work"})
    assert (after["claudeBin"], after["seal"], after["sealBroken"]) == ("", "ok", False)


def test_with_no_keychain_the_stored_values_stand(prefs_file):
    import keyring
    from keyring.backends import fail

    from app.core import prefs, seal

    before = keyring.get_keyring()
    keyring.set_keyring(fail.Keyring())
    seal.reset()
    prefs_file.write_text(WRITTEN_BY_0_26_0)
    got = prefs.load()
    assert (got["agentsAllowed"], got["claudeBin"], got["seal"], got["sealBroken"]) == (False, "/opt/stand-in/claude", "unavailable", False)
    prefs.update({"claudeFolder": "/tmp/work"})
    assert "seal" not in json.loads(prefs_file.read_text())
    keyring.set_keyring(before)
    seal.reset()


def test_the_agent_settings_report_the_seal(keychain, prefs_file):
    from app.services import claude_code

    got = claude_code.agent_prefs(claudeBin="/opt/stand-in/claude")
    assert (got["claudeBin"], got["seal"], got["sealBroken"]) == ("/opt/stand-in/claude", "ok", False)


def test_blocked_blocks_the_in_app_agents_too(prefs_file):
    from app.core import prefs
    from app.routers import mcp
    from app.services import agents

    prefs.update({"agentsAllowed": False})
    for inapp in (True, False):
        out = asyncio.run(mcp.call_tool("pattern_get", {}, inapp=inapp))
        assert out["isError"] is True
        assert out["content"][0]["text"] == agents.SWITCHED_OFF
    prefs.update({"agentsAllowed": True})
