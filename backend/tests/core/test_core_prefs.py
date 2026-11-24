import json

import pytest

from conftest import window_headers

from app.core import prefs

from .support import data_folder, no_keychain

ROUTE = "/api/commands/agents.prefs"


def stored(folder):
    return json.loads((folder / "preferences.json").read_text())


def test_with_nothing_saved_the_preferences_read_as_the_documented_defaults(data_folder, no_keychain):
    got = prefs.load()
    assert got["claudeFolder"] == "" and got["claudeBin"] == "" and got["claudeFirstMessage"] == ""
    assert got["agentsAllowed"] is True


def test_a_saved_preference_reads_back_and_is_in_the_file(data_folder, no_keychain):
    prefs.update({"claudeFolder": "/work/shirts", "claudeFirstMessage": "hello"})
    again = prefs.load()
    assert again["claudeFolder"] == "/work/shirts" and again["claudeFirstMessage"] == "hello"
    assert stored(data_folder)["claudeFolder"] == "/work/shirts"


def test_saving_one_preference_leaves_the_others_as_they_were(data_folder, no_keychain):
    prefs.update({"claudeFolder": "/a", "agentsAllowed": False})
    prefs.update({"claudeFirstMessage": "note"})
    got = prefs.load()
    assert (got["claudeFolder"], got["agentsAllowed"], got["claudeFirstMessage"]) == ("/a", False, "note")


def test_a_preference_can_be_set_back_after_being_changed(data_folder, no_keychain):
    prefs.update({"agentsAllowed": False})
    prefs.update({"agentsAllowed": True})
    assert prefs.load()["agentsAllowed"] is True


@pytest.mark.parametrize("values", [{"claudeFolder": 5}, {"agentsAllowed": "yes"}, {"agentsAllowed": 1}, {"claudeBin": ["x"]}, {"claudeFirstMessage": True}])
def test_a_preference_of_the_wrong_kind_is_refused_and_nothing_is_saved(data_folder, no_keychain, values):
    with pytest.raises(ValueError, match="must be"):
        prefs.update(values)
    assert not (data_folder / "preferences.json").exists()


def test_the_refusal_names_the_preference_and_the_kind_it_wants(data_folder, no_keychain):
    with pytest.raises(ValueError) as caught:
        prefs.update({"agentsAllowed": "no"})
    assert str(caught.value) == "preference agentsAllowed must be bool"


def test_unknown_preferences_are_refused_by_name_in_sorted_order(data_folder, no_keychain):
    with pytest.raises(ValueError) as caught:
        prefs.update({"zeta": 1, "alpha": 2})
    assert str(caught.value) == "unknown preference alpha, zeta"


@pytest.mark.parametrize("text", ["not json at all", "[1, 2]", "\"text\"", "", "null"])
def test_a_damaged_preferences_file_reads_as_the_defaults(data_folder, no_keychain, text):
    data_folder.mkdir(parents=True)
    (data_folder / "preferences.json").write_text(text)
    assert prefs.load()["agentsAllowed"] is True
    assert prefs.load()["claudeFolder"] == ""


def test_saving_over_a_damaged_file_repairs_it(data_folder, no_keychain):
    data_folder.mkdir(parents=True)
    (data_folder / "preferences.json").write_text("{broken")
    prefs.update({"claudeFolder": "/fixed"})
    assert stored(data_folder)["claudeFolder"] == "/fixed"
    assert prefs.load()["claudeFolder"] == "/fixed"


def test_changing_what_load_returns_does_not_change_the_defaults(data_folder, no_keychain):
    prefs.load()["claudeFolder"] = "/scribble"
    assert prefs.load()["claudeFolder"] == ""
    assert prefs.DEFAULTS["claudeFolder"] == ""


def test_saving_leaves_no_temporary_file_behind(data_folder, no_keychain):
    prefs.update({"claudeFolder": "/x"})
    assert sorted(p.name for p in data_folder.iterdir() if p.name.startswith("preferences")) == ["preferences.json"]


def test_preferences_follow_the_data_folder(tmp_path, monkeypatch, no_keychain):
    monkeypatch.setenv("FORMFABRIC_DATA_DIR", str(tmp_path / "one"))
    prefs.update({"claudeFolder": "/in-one"})
    monkeypatch.setenv("FORMFABRIC_DATA_DIR", str(tmp_path / "two"))
    assert prefs.load()["claudeFolder"] == ""
    monkeypatch.setenv("FORMFABRIC_DATA_DIR", str(tmp_path / "one"))
    assert prefs.load()["claudeFolder"] == "/in-one"


def test_the_window_saves_and_reads_back_preferences_through_the_command_route(client, data_folder, no_keychain):
    r = client.post(ROUTE, json={"claudeFolder": "/via/route", "agentsAllowed": False})
    assert r.status_code == 200 and r.json()["result"]["claudeFolder"] == "/via/route"
    read = client.post(ROUTE, json={}).json()["result"]
    assert read["claudeFolder"] == "/via/route" and read["agentsAllowed"] is False
    assert stored(data_folder)["agentsAllowed"] is False
    client.post(ROUTE, json={"agentsAllowed": True})


def test_the_command_route_answers_400_with_the_reason_for_a_wrong_kind(client, data_folder, no_keychain):
    r = client.post(ROUTE, json={"claudeFolder": 5})
    assert r.status_code == 400
    assert r.json() == {"ok": False, "command": "agents.prefs", "error": "agents.prefs: claudeFolder must be a string"}


def test_the_command_route_refuses_a_preference_it_does_not_know(client, data_folder, no_keychain):
    r = client.post(ROUTE, json={"nope": 5})
    assert r.status_code == 400 and r.json()["error"] == "agents.prefs does not take nope"


def test_without_the_window_key_preferences_cannot_be_changed_and_stay_unchanged(bare, data_folder, no_keychain):
    r = bare.post(ROUTE, json={"claudeFolder": "/sneaky"})
    assert r.status_code == 403
    assert prefs.load()["claudeFolder"] == ""


def test_the_command_route_reports_the_seal_as_unavailable_without_a_keychain(client, data_folder, no_keychain):
    result = client.post(ROUTE, json={}).json()["result"]
    assert (result["seal"], result["sealBroken"], result["sealWasBroken"]) == ("unavailable", False, False)
