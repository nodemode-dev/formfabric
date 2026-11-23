import json
import math

import pytest

from .support import CATALOGUE, SPECS, Fail, FakeEngine, client, engine, engine_module, no_default_client  # noqa: F401

import formfabric  # noqa: E402
from formfabric import commands as c  # noqa: E402
from formfabric.errors import ArgumentError, CommandError  # noqa: E402


def test_every_catalogue_command_has_a_function_named_with_underscores():
    assert len(c.COMMANDS) == len(CATALOGUE["commands"])
    for spec in CATALOGUE["commands"]:
        fn = getattr(c, spec["id"].replace(".", "_"))
        assert fn is c.COMMANDS[spec["id"]]
        assert fn.command == spec["id"]


def test_dotted_and_underscore_names_are_the_same_function():
    assert c.sample.load is c.sample_load
    assert c.sim.step is c.sim_step
    assert c.pattern.rectangle is c.pattern_rectangle


def test_snapshot_names_match_the_mcp_names_with_dots_as_underscores():
    for spec in CATALOGUE["commands"]:
        assert spec["name"] == spec["id"].replace(".", "_")


def test_python_keyword_argument_gets_a_trailing_underscore_and_is_sent_without_it(client, engine):
    c.colorway_create(name="red", from_=3, client=client)
    assert engine.args_of("colorway.create") == [{"name": "red", "from": 3}]


def test_every_keyword_named_argument_is_exposed_with_trailing_underscore():
    import inspect
    import keyword

    seen = 0
    for spec in CATALOGUE["commands"]:
        for key in spec["inputSchema"]["properties"]:
            if keyword.iskeyword(key):
                assert key + "_" in inspect.signature(c.COMMANDS[spec["id"]]).parameters
                seen += 1
    assert seen > 0


def test_function_arguments_are_keyword_only():
    with pytest.raises(TypeError):
        c.sim_step(5)


def test_sim_step_sends_the_frames_given(client, engine):
    c.sim_step(frames=10, client=client)
    assert engine.calls == [("sim.step", {"frames": 10})]


def test_arguments_left_out_are_not_sent(client, engine):
    c.sample_load(name="tshirt", client=client)
    assert engine.args_of("sample.load") == [{"name": "tshirt"}]


def test_command_result_is_returned(client, engine):
    engine.replies["sim.state"] = {"frame": 7, "running": False}
    assert c.sim_state(client=client) == {"frame": 7, "running": False}


def test_default_client_is_used_when_none_is_given(client, engine, no_default_client):
    formfabric.use(client)
    c.sim_step(frames=2)
    assert engine.names() == ["sim.step"]


def test_explicit_client_wins_over_default(engine, client, no_default_client, monkeypatch):
    other = FakeEngine()
    import sys

    monkeypatch.setitem(sys.modules, "ffb_engine", engine_module(other))
    second = formfabric.connect("engine")
    formfabric.use(client)
    c.sim_step(frames=1, client=second)
    assert other.names() == ["sim.step"]
    assert engine.names() == []


@pytest.mark.parametrize("frames", [0, -3, 2001])
def test_sim_step_frames_outside_range_are_refused_and_nothing_is_sent(client, engine, frames):
    with pytest.raises(ArgumentError):
        c.sim_step(frames=frames, client=client)
    assert engine.calls == []


def test_sim_step_frames_at_both_limits_are_accepted(client, engine):
    c.sim_step(frames=1, client=client)
    c.sim_step(frames=2000, client=client)
    assert [a["frames"] for a in engine.args_of("sim.step")] == [1, 2000]


@pytest.mark.parametrize("bad", ["3", True, [1], 2.5])
def test_sim_step_frames_of_the_wrong_type_are_refused(client, engine, bad):
    with pytest.raises(ArgumentError):
        c.sim_step(frames=bad, client=client)
    assert engine.calls == []


def test_none_is_refused_with_advice_to_leave_it_out(client, engine):
    with pytest.raises(ArgumentError, match="leave it out"):
        c.sim_step(frames=None, client=client)


def test_unknown_argument_name_is_refused(client, engine):
    with pytest.raises(ArgumentError, match="unknown argument bogus"):
        c.sim_step(bogus=1, client=client)
    assert engine.calls == []


def test_missing_required_argument_is_refused_naming_it(client, engine):
    with pytest.raises(ArgumentError, match="missing required argument name"):
        c.sample_load(client=client)
    assert engine.calls == []


def test_enum_violation_lists_the_allowed_values(client, engine):
    with pytest.raises(ArgumentError) as err:
        c.sample_load(name="hoodie", client=client)
    for option in ("tshirt", "skirt", "dress"):
        assert option in str(err.value)
    assert err.value.command == "sample.load"


def test_refusal_message_starts_with_the_command_id(client):
    with pytest.raises(ArgumentError) as err:
        c.sim_step(frames=0, client=client)
    assert str(err.value).startswith("sim.step: ")
    assert "at least 1" in str(err.value)


def test_argument_error_is_also_a_value_error():
    assert issubclass(ArgumentError, ValueError)
    assert issubclass(ArgumentError, formfabric.FormfabricError)


@pytest.mark.parametrize(
    "kwargs",
    [
        {"panel": 1, "name": ""},
        {"panel": 1, "name": "x" * 65},
        {"panel": 0, "name": "ok"},
        {"panel": 1.5, "name": "ok"},
    ],
)
def test_rename_limits_on_length_and_id_are_enforced(client, engine, kwargs):
    with pytest.raises(ArgumentError):
        c.pattern_rename(client=client, **kwargs)
    assert engine.calls == []


def test_rename_accepts_name_of_exactly_64_characters(client, engine):
    c.pattern_rename(panel=1, name="x" * 64, client=client)
    assert engine.args_of("pattern.rename")[0]["name"] == "x" * 64


def test_rectangle_width_bounds_are_the_catalogue_limits(client, engine):
    low = SPECS["pattern.rectangle"]["inputSchema"]["properties"]["width"]
    c.pattern_rectangle(width=low["minimum"], height=10, client=client)
    c.pattern_rectangle(width=low["maximum"], height=10, client=client)
    with pytest.raises(ArgumentError):
        c.pattern_rectangle(width=low["minimum"] / 2, height=10, client=client)
    with pytest.raises(ArgumentError):
        c.pattern_rectangle(width=low["maximum"] * 2, height=10, client=client)
    assert len(engine.args_of("pattern.rectangle")) == 2


def test_rectangle_width_as_text_is_refused(client):
    with pytest.raises(ArgumentError, match="must be number"):
        c.pattern_rectangle(width="400", height=300, client=client)


@pytest.mark.xfail(strict=True, reason="DEFECT: pattern_rectangle(width=nan) is sent to the engine because nan passes every minimum and maximum comparison; README says arguments are checked against minimum and maximum and refused with ArgumentError, and nan is within no range")
def test_not_a_number_width_is_refused(client, engine):
    with pytest.raises(ArgumentError):
        c.pattern_rectangle(width=math.nan, height=300, client=client)


def test_infinite_height_is_refused(client, engine):
    with pytest.raises(ArgumentError):
        c.pattern_rectangle(width=10, height=math.inf, client=client)
    assert engine.calls == []


def test_array_with_too_few_items_is_refused(client, engine):
    with pytest.raises(ArgumentError, match="at least 1"):
        c.pattern_delete(panels=[], client=client)
    with pytest.raises(ArgumentError, match="at least 3"):
        c.pattern_draw_panel(points=[[0, 0], [1, 1]], client=client)
    assert engine.calls == []


def test_array_items_are_checked_one_by_one(client, engine):
    with pytest.raises(ArgumentError, match=r"panels\[1\]"):
        c.pattern_delete(panels=[3, 0], client=client)
    with pytest.raises(ArgumentError, match=r"panels\[0\]"):
        c.pattern_duplicate(panels=["a"], client=client)


def test_array_with_too_many_items_is_refused(client):
    with pytest.raises(ArgumentError, match="at most 2"):
        c.pattern_duplicate(panels=[1], offset=[1, 2, 3], client=client)


def test_string_pattern_in_the_schema_is_enforced(client, engine):
    with pytest.raises(ArgumentError, match="must match"):
        c.link_start(folder="/tmp/x", file="bad name!", client=client)
    c.link_start(folder="/tmp/x", file="good_name-1", client=client)
    assert engine.args_of("link.start")[0]["file"] == "good_name-1"


def test_argument_errors_come_before_any_send_even_for_valid_other_fields(client, engine):
    with pytest.raises(ArgumentError):
        c.export_run(format="docx", path="/tmp/a.docx", client=client)
    assert engine.calls == []


def test_engine_refusal_becomes_command_error_with_its_code_and_message(client, engine):
    engine.replies["sample.load"] = Fail("unsaved_changes", "the project has unsaved edits")
    with pytest.raises(CommandError) as err:
        c.sample_load(name="skirt", client=client)
    assert err.value.code == "unsaved_changes"
    assert err.value.message == "the project has unsaved edits"
    assert err.value.command == "sample.load"
    assert "unsaved_changes" in str(err.value)


def test_engine_error_given_as_plain_text_gets_the_failed_code(client, engine):
    engine.dispatch = lambda name, raw: json.dumps({"ok": False, "error": "no body loaded"})
    with pytest.raises(CommandError) as err:
        c.sim_reset(client=client)
    assert (err.value.code, err.value.message) == ("failed", "no body loaded")


def test_docstring_carries_description_and_argument_list():
    doc = c.sim_step.__doc__
    spec = SPECS["sim.step"]
    assert spec["description"].strip() in doc
    assert doc.startswith(f"{spec['label']} (sim.step, MCP tool sim_step")
    assert "frames (integer, optional, minimum 1, maximum 2000, default 1)" in doc
    assert "Pass client=Client" in doc


def test_docstring_marks_required_arguments_and_enum_choices():
    doc = c.sample_load.__doc__
    assert "name (string, required, one of 'tshirt', 'skirt', 'dress')" in doc


def test_signature_lists_required_arguments_without_defaults_and_a_client_option():
    import inspect

    params = inspect.signature(c.sample_load).parameters
    assert params["name"].default is inspect.Parameter.empty
    assert params["force"].default is c.UNSET
    assert params["client"].default is None
    assert all(p.kind is inspect.Parameter.KEYWORD_ONLY for p in params.values())


def test_every_function_signature_has_exactly_the_schema_properties_plus_client():
    import inspect

    for spec in CATALOGUE["commands"]:
        names = set(inspect.signature(c.COMMANDS[spec["id"]]).parameters)
        expected = {k + "_" if __import__("keyword").iskeyword(k) else k for k in spec["inputSchema"]["properties"]}
        assert names == expected | {"client"}, spec["id"]


def test_every_command_without_required_arguments_dispatches_under_its_id(client, engine):
    ids = [s["id"] for s in CATALOGUE["commands"] if not s["inputSchema"].get("required")]
    for command in ids:
        client.call(command)
    assert engine.names() == ids


def test_every_command_with_required_arguments_refuses_an_empty_call(client, engine):
    needing = [s["id"] for s in CATALOGUE["commands"] if s["inputSchema"].get("required")]
    assert needing
    for command in needing:
        with pytest.raises(ArgumentError, match="missing required argument"):
            client.call(command)
    assert engine.calls == []


def test_every_command_refuses_an_argument_it_does_not_have(client, engine):
    for spec in CATALOGUE["commands"]:
        required = spec["inputSchema"].get("required") or []
        if required:
            continue
        with pytest.raises(ArgumentError, match="unknown argument"):
            client.call(spec["id"], zzz_not_an_argument=1)
    assert engine.calls == []
