import pytest

from .support import Fail, FakeEngine, client, engine, no_default_client  # noqa: F401

import formfabric  # noqa: E402
from formfabric import Garment, Link, Panel, Sim, sample_drape  # noqa: E402
from formfabric.errors import ArgumentError, CommandError  # noqa: E402


def test_sample_loads_by_name_with_force_and_reads_the_panels(client, engine):
    g = Garment.sample("skirt", client=client)
    assert engine.args_of("sample.load") == [{"name": "skirt", "force": True}]
    assert [p.name for p in g.panels] == ["Front", "Back"]
    assert [p.id for p in g.panels] == [1, 2]


def test_sample_passes_the_body_when_given_and_not_otherwise(client, engine):
    Garment.sample("dress", body="MPFB2 Male", client=client)
    Garment.sample("dress", body=0, client=client)
    Garment.sample("dress", client=client, force=False)
    bodies = engine.args_of("sample.load")
    assert bodies[0]["body"] == "MPFB2 Male"
    assert bodies[1]["body"] == 0
    assert "body" not in bodies[2] and bodies[2]["force"] is False


def test_sample_with_an_unknown_name_is_refused_listing_the_samples(client, engine):
    with pytest.raises(ValueError) as err:
        Garment.sample("hoodie", client=client)
    assert "tshirt, skirt, dress" in str(err.value)
    assert engine.calls == []


def test_sample_default_name_is_tshirt(client, engine):
    Garment.sample(client=client)
    assert engine.args_of("sample.load")[0]["name"] == "tshirt"


def test_sample_unsaved_changes_refusal_reaches_the_caller(client, engine):
    engine.replies["sample.load"] = Fail("unsaved_changes", "confirm first")
    with pytest.raises(CommandError) as err:
        Garment.sample("tshirt", client=client, force=False)
    assert err.value.code == "unsaved_changes"


def test_new_starts_an_empty_project(client, engine):
    Garment.sample("tshirt", client=client)
    g = Garment.new(client)
    assert engine.names()[-1] == "file.new"
    assert g.panels == []


def test_open_and_save_send_the_path(client, engine, tmp_path):
    target = str(tmp_path / "a.ffbrc")
    g = Garment.open(target, client=client)
    g.save(target)
    assert engine.args_of("file.open") == [{"path": target}]
    assert engine.args_of("file.save") == [{"path": target}]


def test_garment_uses_the_default_client_when_none_is_given(client, engine, no_default_client):
    formfabric.use(client)
    g = Garment.new()
    assert g.client is client


def test_rectangle_sends_geometry_and_returns_the_new_panel(client, engine):
    g = Garment.new(client)
    panel = g.rectangle(400, 300, name="swatch", x=10, y=-5)
    assert engine.args_of("pattern.rectangle") == [{"x": 10, "y": -5, "width": 400, "height": 300, "name": "swatch"}]
    assert isinstance(panel, Panel)
    assert panel.name == "swatch"
    assert g.panel("swatch").id == panel.id == 1


def test_rectangle_extra_options_are_passed_through_and_checked(client, engine):
    g = Garment.new(client)
    g.rectangle(10, 10, corner_radius=2.5)
    assert engine.args_of("pattern.rectangle")[0]["corner_radius"] == 2.5
    with pytest.raises(ArgumentError):
        g.rectangle(10, 10, colour="red")


def test_rectangle_with_zero_width_is_refused_before_sending(client, engine):
    g = Garment.new(client)
    with pytest.raises(ArgumentError):
        g.rectangle(0, 10)
    assert "pattern.rectangle" not in engine.names()


def test_two_rectangles_get_different_ids_and_are_listed_in_order(client, engine):
    g = Garment.new(client)
    a = g.rectangle(10, 10, name="a")
    b = g.rectangle(20, 20, name="b")
    assert a.id != b.id
    assert [p.name for p in g.panels] == ["a", "b"]


def test_draw_converts_points_to_lists_and_returns_the_panel(client, engine):
    g = Garment.new(client)
    panel = g.draw([(0, 0), (100, 0), (50, 80)], name="tri")
    assert engine.args_of("pattern.draw_panel") == [{"points": [[0, 0], [100, 0], [50, 80]], "name": "tri"}]
    assert panel.name == "tri"


def test_draw_with_two_points_is_refused(client, engine):
    g = Garment.new(client)
    with pytest.raises(ArgumentError, match="at least 3"):
        g.draw([(0, 0), (1, 1)])
    assert "pattern.draw_panel" not in engine.names()


def test_panel_lookup_by_id_and_by_name(client, engine):
    g = Garment.sample("tshirt", client=client)
    assert g.panel(2).name == "Back"
    assert g.panel("Front").id == 1


def test_panel_lookup_of_missing_panel_raises_key_error(client, engine):
    g = Garment.sample("tshirt", client=client)
    with pytest.raises(KeyError, match="no panel 'Sleeve'"):
        g.panel("Sleeve")
    with pytest.raises(KeyError):
        g.panel(99)


def test_panels_property_returns_a_copy(client, engine):
    g = Garment.sample("tshirt", client=client)
    g.panels.clear()
    assert len(g.panels) == 2


def test_panel_rename_sends_id_and_name_and_updates_the_object(client, engine):
    g = Garment.sample("tshirt", client=client)
    p = g.panel("Front")
    assert p.rename("Chest") is p
    assert engine.args_of("pattern.rename") == [{"panel": 1, "name": "Chest"}]
    assert p.name == "Chest"


def test_panel_rename_to_empty_name_is_refused_and_keeps_old_name(client, engine):
    g = Garment.sample("tshirt", client=client)
    p = g.panel("Front")
    with pytest.raises(ArgumentError):
        p.rename("")
    assert p.name == "Front"


def test_panel_measure_and_positions_send_the_panel_id(client, engine):
    engine.replies["pattern.measure"] = {"area": 12.0}
    engine.replies["sim.positions"] = {"positions": [[0, 0, 0], [1, 0, 0]]}
    g = Garment.sample("tshirt", client=client)
    p = g.panel(2)
    assert p.measure() == {"area": 12.0}
    assert p.positions(decimate=4) == [[0, 0, 0], [1, 0, 0]]
    assert engine.args_of("pattern.measure") == [{"panel": 2}]
    assert engine.args_of("sim.positions") == [{"panel": 2, "decimate": 4}]


def test_panel_positions_accepts_a_bare_list_reply(client, engine):
    engine.replies["sim.positions"] = [[1, 2, 3]]
    g = Garment.sample("tshirt", client=client)
    assert g.panel(1).positions() == [[1, 2, 3]]


def test_panel_positions_decimate_zero_is_refused(client, engine):
    g = Garment.sample("tshirt", client=client)
    with pytest.raises(ArgumentError):
        g.panel(1).positions(decimate=0)


def test_panel_duplicate_returns_the_new_panel_not_the_original(client, engine):
    g = Garment.sample("tshirt", client=client)
    copy = g.panel("Front").duplicate()
    assert copy.id == 3
    assert copy.name == "Front copy"
    assert len(g.panels) == 3
    assert engine.args_of("pattern.duplicate") == [{"panels": [1]}]


def test_panel_delete_removes_it_from_the_garment(client, engine):
    g = Garment.sample("tshirt", client=client)
    g.panel("Back").delete()
    assert engine.args_of("pattern.delete") == [{"panels": [2]}]
    assert [p.name for p in g.panels] == ["Front"]


def test_sew_accepts_panels_or_ids_and_maps_to_the_command_arguments(client, engine):
    g = Garment.sample("tshirt", client=client)
    g.sew((g.panel("Front"), 2), (3, 0), fold="flat")
    assert engine.args_of("sew.segment") == [{"panel_a": 1, "segment_a": 2, "panel_b": 3, "segment_b": 0, "fold": "flat"}]


def test_sew_with_an_unknown_fold_is_refused(client, engine):
    g = Garment.sample("tshirt", client=client)
    with pytest.raises(ArgumentError):
        g.sew((1, 0), (2, 0), fold="zigzag")
    assert "sew.segment" not in engine.names()


def test_undo_and_redo_refresh_the_panel_list(client, engine):
    g = Garment.sample("tshirt", client=client)
    engine.replies["history.undo"] = lambda a: engine.panels.pop() and {}
    g.undo()
    assert [p.name for p in g.panels] == ["Front"]
    assert engine.names()[-2:] == ["history.undo", "pattern.get"]
    g.redo()
    assert engine.names()[-2:] == ["history.redo", "pattern.get"]


@pytest.mark.parametrize(
    "path, kind",
    [("/tmp/a.glb", "glb"), ("/tmp/a.OBJ", "obj"), ("/tmp/a.gltf", "gltf"), ("/tmp/a.usda", "usd"), ("/tmp/a.usd", "usd"), ("/tmp/a.fbx", "fbx"), ("/tmp/v1.2/a.abc", "abc")],
)
def test_export_takes_the_format_from_the_file_extension(client, engine, path, kind):
    Garment.new(client).export(path)
    assert engine.args_of("export.run") == [{"format": kind, "path": path}]


def test_export_explicit_format_overrides_the_extension(client, engine):
    Garment.new(client).export("/tmp/out.bin", "fbx", binary=True)
    assert engine.args_of("export.run") == [{"format": "fbx", "path": "/tmp/out.bin", "binary": True}]


def test_export_glb_forces_glb_and_forwards_options(client, engine):
    Garment.new(client).export_glb("/tmp/x.out", units="m")
    assert engine.args_of("export.run") == [{"format": "glb", "path": "/tmp/x.out", "units": "m"}]


def test_export_to_an_unsupported_format_is_refused_naming_the_choices(client, engine):
    with pytest.raises(ArgumentError) as err:
        Garment.new(client).export("/tmp/a.docx")
    assert "'glb'" in str(err.value)
    assert "export.run" not in engine.names()


def test_export_without_extension_is_refused(client, engine):
    with pytest.raises(ArgumentError):
        Garment.new(client).export("/tmp/noextension")


def test_sim_reset_step_start_stop_state_map_to_their_commands(client, engine):
    sim = Garment.new(client).sim
    assert isinstance(sim, Sim)
    sim.reset()
    sim.step()
    sim.step(7)
    sim.start()
    sim.stop()
    sim.state()
    assert engine.calls[1:] == [("sim.reset", {}), ("sim.step", {"frames": 1}), ("sim.step", {"frames": 7}), ("sim.start", {}), ("sim.stop", {}), ("sim.state", {})]


def test_sim_step_zero_frames_is_refused(client, engine):
    with pytest.raises(ArgumentError):
        Sim(client).step(0)
    assert engine.calls == []


def test_drape_resets_steps_and_returns_the_state(client, engine):
    state = Sim(client).drape(60)
    assert engine.names() == ["sim.reset", "sim.step", "sim.state"]
    assert engine.args_of("sim.step") == [{"frames": 60}]
    assert state == {"frame": 60}


def test_drape_without_reset_continues_from_the_current_frame(client, engine):
    sim = Sim(client)
    sim.step(10)
    state = sim.drape(5, reset=False)
    assert "sim.reset" not in engine.names()
    assert state == {"frame": 15}


def test_drape_default_is_ninety_frames(client, engine):
    Sim(client).drape()
    assert engine.args_of("sim.step") == [{"frames": 90}]


@pytest.mark.parametrize("name", ["fast", "normal", "quality", "final"])
def test_preset_sends_the_quality_name_through_settings(client, engine, name):
    Sim(client).preset(name)
    assert engine.args_of("sim.settings") == [{"preset": name}]


def test_preset_with_an_unknown_name_is_refused(client, engine):
    with pytest.raises(ValueError, match="fast, normal, quality, final"):
        Sim(client).preset("ultra")
    assert engine.calls == []


def test_settings_forwards_values_and_checks_their_range(client, engine):
    Sim(client).settings(air_damping=0.5, cg_iterations=20)
    assert engine.args_of("sim.settings") == [{"air_damping": 0.5, "cg_iterations": 20}]
    with pytest.raises(ArgumentError):
        Sim(client).settings(air_damping=1.5)


def test_wait_returns_once_the_frame_is_reached(client, engine):
    frames = iter([1, 3, 5, 9])
    engine.replies["sim.state"] = lambda a: {"frame": next(frames)}
    state = Sim(client).wait(5)
    assert state == {"frame": 5}
    assert len(engine.args_of("sim.state")) == 3


def test_wait_gives_up_at_the_timeout_and_returns_the_last_state(client, engine):
    engine.replies["sim.state"] = {"frame": 2}
    assert Sim(client).wait(100, timeout=0.0) == {"frame": 2}


def test_link_start_sends_folder_usd_format_and_cadence(client, engine, tmp_path):
    link = Garment.new(client).link(str(tmp_path), every_frames=5, file="demo")
    assert isinstance(link, Link)
    assert engine.args_of("link.start") == [{"folder": str(tmp_path), "every_frames": 5, "format": "usd", "file": "demo"}]


def test_link_as_context_manager_stops_on_exit_even_after_an_error(client, engine, tmp_path):
    with pytest.raises(RuntimeError):
        with Garment.new(client).link(str(tmp_path)):
            raise RuntimeError("script failed")
    assert engine.names()[-1] == "link.stop"


def test_link_state_and_stop(client, engine):
    link = Link(client)
    link.state()
    link.stop()
    assert engine.names() == ["link.state", "link.stop"]


def test_link_with_zero_cadence_is_refused_and_never_started(client, engine, tmp_path):
    with pytest.raises(ArgumentError):
        Garment.new(client).link(str(tmp_path), every_frames=0)
    assert "link.start" not in engine.names()


def test_sample_drape_loads_resets_steps_and_returns_the_garment(client, engine):
    g = sample_drape("dress", frames=30, client=client)
    assert isinstance(g, Garment)
    assert engine.names() == ["sample.load", "pattern.get", "sim.reset", "sim.step", "sim.state"]
    assert engine.args_of("sim.step") == [{"frames": 30}]


def test_sample_drape_defaults_are_tshirt_and_sixty_frames(client, engine):
    sample_drape(client=client)
    assert engine.args_of("sample.load")[0]["name"] == "tshirt"
    assert engine.args_of("sim.step") == [{"frames": 60}]


def test_sample_drape_passes_the_body_choice(client, engine):
    sample_drape("skirt", body="MPFB2 Female", client=client)
    assert engine.args_of("sample.load")[0]["body"] == "MPFB2 Female"


def test_sample_drape_with_unknown_sample_sends_nothing(client, engine):
    with pytest.raises(ValueError):
        sample_drape("cape", client=client)
    assert engine.calls == []


def test_garment_repr_names_panel_count_and_backend(client, engine):
    g = Garment.sample("tshirt", client=client)
    assert repr(g) == "Garment(panels=2, backend=engine)"
    assert repr(g.panel(1)) == "Panel(id=1, name='Front')"
