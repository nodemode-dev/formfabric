import json
import os
from pathlib import Path

import pytest

pytestmark = pytest.mark.skipif(os.name == "nt", reason="the stand-in agy is a POSIX shell script")


def entry(folder: Path) -> dict:
    data = json.loads((folder / ".agents" / "mcp_config.json").read_text(encoding="utf-8"))
    return data["mcpServers"]["formfabric_tab"]


def test_two_antigravity_tabs_each_keep_their_own_entry():
    from app.services import agent_programs, agents, terminal

    first = agent_programs.open_terminal("antigravity", 80, 24)
    second = agent_programs.open_terminal("antigravity", 80, 24)
    try:
        one, two = Path(first["cwd"]), Path(second["cwd"])
        assert one != two
        assert one.parent == two.parent == agent_programs.tab_folder("antigravity")
        token_one = entry(one)["headers"][agents.INAPP_HEADER]
        token_two = entry(two)["headers"][agents.INAPP_HEADER]
        assert token_one != token_two
        assert agents.is_inapp(token_one) and agents.is_inapp(token_two)
        if os.name != "nt":
            assert (one / ".agents" / "mcp_config.json").stat().st_mode & 0o777 == 0o600

        terminal.close(second["id"])
        assert not two.exists()
        assert not agents.is_inapp(token_two)
        assert entry(one)["headers"][agents.INAPP_HEADER] == token_one
        assert agents.is_inapp(token_one)
    finally:
        terminal.close(first["id"])
        terminal.close(second["id"])
    assert not Path(first["cwd"]).exists()


def test_a_tab_that_fails_to_start_leaves_no_folder(monkeypatch):
    from app.services import agent_programs, terminal

    before = set(agent_programs.tab_folder("antigravity").iterdir())

    def refuse(*args, **kwargs):
        raise OSError("no terminal")

    monkeypatch.setattr(terminal, "open_session", refuse)
    with pytest.raises(OSError):
        agent_programs.open_terminal("antigravity", 80, 24)
    assert set(agent_programs.tab_folder("antigravity").iterdir()) == before
