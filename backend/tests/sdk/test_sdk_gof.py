import json

import pytest

from .support import FakeEngine, client, clean_env, engine  # noqa: F401

import formfabric  # noqa: E402
from formfabric import gof  # noqa: E402
from formfabric.errors import CommandError  # noqa: E402


def writing_engine(engine, vertices=120, triangles=200):
    def export(args):
        with open(args["path"], "wb") as handle:
            handle.write(b"mesh")
        return {"vertices": vertices, "triangles": triangles}

    engine.replies["export.run"] = export


def manifest(folder):
    return json.loads((folder / "gof.json").read_text(encoding="utf-8"))


def test_send_exports_a_glb_in_metres_y_up_into_the_folder(client, engine, tmp_path):
    writing_engine(engine)
    out = gof.send("shirt", "blender", tmp_path, client=client)
    call = engine.args_of("export.run")[0]
    assert call == {"format": "glb", "path": str(tmp_path / "shirt.glb"), "units": "m", "axis": "y_up"}
    assert (tmp_path / "shirt.glb").read_bytes() == b"mesh"
    assert out["path"] == str(tmp_path / "shirt.glb")
    assert out["manifest"] == str(tmp_path / "gof.json")


def test_manifest_entry_has_the_fields_the_handoff_document_lists(client, engine, tmp_path):
    writing_engine(engine, vertices=321, triangles=640)
    gof.send("shirt", "blender", tmp_path, client=client)
    data = manifest(tmp_path)
    assert data["gof"] == 1
    assert data["app"] == "formfabric"
    assert data["up_axis"] == "y"
    entry = data["objects"][0]
    assert entry["name"] == "shirt" and entry["file"] == "shirt.glb"
    assert entry["format"] == "glb" and entry["target"] == "blender"
    assert entry["vertices"] == 321 and entry["triangles"] == 640
    assert entry["returns"] is False and entry["app"] == "formfabric"
    assert entry["revision"] == 1
    assert isinstance(entry["sent"], int) and entry["sent"] > 1_600_000_000


def test_first_entry_id_is_in_the_reserved_range_from_two_million(client, engine, tmp_path):
    writing_engine(engine)
    gof.send("a", "any", tmp_path, client=client)
    assert manifest(tmp_path)["objects"][0]["id"] == 2_000_000


@pytest.mark.parametrize("target, kind, units, per_mm", [("blender", "glb", "m", 0.001), ("unreal", "glb", "m", 0.001), ("formsurface", "glb", "m", 0.001), ("any", "glb", "m", 0.001), ("unity", "fbx", "cm", 0.1)])
def test_scale_turns_millimetres_into_the_file_unit_for_each_target(client, engine, tmp_path, target, kind, units, per_mm):
    writing_engine(engine)
    gof.send("w", target, tmp_path, client=client)
    entry = manifest(tmp_path)["objects"][0]
    assert (entry["format"], entry["units"]) == (kind, units)
    expected_in_unit = {"m": 1.0, "cm": 100.0}[units]
    one_metre_in_mm = 1000.0
    assert one_metre_in_mm * entry["scale"] == pytest.approx(expected_in_unit)
    assert entry["scale"] == pytest.approx(per_mm)
    assert engine.args_of("export.run")[0]["units"] == units
    assert engine.args_of("export.run")[0]["format"] == kind


def test_sending_the_same_name_again_bumps_the_revision_and_keeps_the_id(client, engine, tmp_path):
    writing_engine(engine)
    gof.send("shirt", "any", tmp_path, client=client)
    gof.send("shirt", "any", tmp_path, client=client)
    gof.send("shirt", "any", tmp_path, client=client)
    objects = manifest(tmp_path)["objects"]
    assert len(objects) == 1
    assert objects[0]["revision"] == 3 and objects[0]["id"] == 2_000_000


def test_a_second_garment_gets_the_next_id_and_leaves_the_first_alone(client, engine, tmp_path):
    writing_engine(engine)
    gof.send("shirt", "any", tmp_path, client=client)
    first = manifest(tmp_path)["objects"][0]
    gof.send("skirt", "any", tmp_path, client=client)
    objects = manifest(tmp_path)["objects"]
    assert [o["id"] for o in objects] == [2_000_000, 2_000_001]
    assert objects[0] == first


def test_entries_written_by_other_apps_and_top_level_keys_are_kept(client, engine, tmp_path):
    writing_engine(engine)
    other = {"id": 7, "name": "bust", "file": "bust.glb", "format": "glb", "target": "any", "revision": 4, "app": "formsculpt"}
    (tmp_path / "gof.json").write_text(json.dumps({"gof": 1, "app": "formsculpt", "version": "1.2.3", "up_axis": "y", "mm_per_unit": 2.5, "objects": [other]}), encoding="utf-8")
    gof.send("shirt", "any", tmp_path, client=client, version="ignored")
    data = manifest(tmp_path)
    assert data["objects"][0] == other
    assert data["objects"][1]["name"] == "shirt"
    assert (data["app"], data["version"], data["mm_per_unit"]) == ("formsculpt", "1.2.3", 2.5)


def test_new_manifest_takes_the_version_given_and_default_scene_size(client, engine, tmp_path):
    writing_engine(engine)
    gof.send("shirt", "any", tmp_path, client=client, version="0.9.9")
    data = manifest(tmp_path)
    assert data["version"] == "0.9.9" and data["mm_per_unit"] == 1.0


def test_an_id_already_taken_in_the_reserved_range_is_not_reused(client, engine, tmp_path):
    writing_engine(engine)
    taken = [{"id": 2_000_000, "file": "x.glb"}, {"id": 2_000_001, "file": "y.glb"}]
    (tmp_path / "gof.json").write_text(json.dumps({"objects": taken}), encoding="utf-8")
    gof.send("shirt", "any", tmp_path, client=client)
    assert manifest(tmp_path)["objects"][-1]["id"] == 2_000_002


@pytest.mark.parametrize("junk", ["not json at all", "[1, 2, 3]", '"text"', ""])
def test_an_unreadable_manifest_is_replaced_by_a_fresh_one(client, engine, tmp_path, junk):
    writing_engine(engine)
    (tmp_path / "gof.json").write_text(junk, encoding="utf-8")
    gof.send("shirt", "any", tmp_path, client=client)
    assert [o["name"] for o in manifest(tmp_path)["objects"]] == ["shirt"]


def test_no_temporary_files_remain_after_sending(client, engine, tmp_path):
    writing_engine(engine)
    gof.send("shirt", "any", tmp_path, client=client)
    gof.send("shirt", "any", tmp_path, client=client)
    assert sorted(p.name for p in tmp_path.iterdir()) == ["gof.json", "shirt.glb"]


def test_the_folder_is_created_when_missing(client, engine, tmp_path):
    writing_engine(engine)
    deep = tmp_path / "a" / "b"
    gof.send("shirt", "any", deep, client=client)
    assert (deep / "gof.json").is_file()


@pytest.mark.parametrize("name, file", [("my shirt!", "my_shirt_.glb"), ("T-shirt v2", "T_shirt_v2.glb"), ("", "garment.glb"), ("tshirt", "tshirt.glb")])
def test_file_name_keeps_only_letters_and_digits(client, engine, tmp_path, name, file):
    writing_engine(engine)
    gof.send(name, "any", tmp_path, client=client)
    assert manifest(tmp_path)["objects"][0]["file"] == file
    assert manifest(tmp_path)["objects"][0]["name"] == name


def test_a_path_in_the_name_cannot_leave_the_folder(client, engine, tmp_path):
    writing_engine(engine)
    gof.send("../../escape", "any", tmp_path / "gof", client=client)
    assert engine.args_of("export.run")[0]["path"].startswith(str(tmp_path / "gof"))
    assert (tmp_path / "gof" / "______escape.glb").is_file()


def test_unknown_target_is_refused_before_anything_is_written(client, engine, tmp_path):
    with pytest.raises(ValueError, match="blender, unreal, formsurface, unity, any"):
        gof.send("shirt", "maya", tmp_path, client=client)
    assert engine.calls == []
    assert list(tmp_path.iterdir()) == []


def test_export_refusal_leaves_the_manifest_untouched(client, engine, tmp_path):
    from .support import Fail

    writing_engine(engine)
    gof.send("shirt", "any", tmp_path, client=client)
    before = (tmp_path / "gof.json").read_text(encoding="utf-8")
    engine.replies["export.run"] = Fail("no_mesh", "drape first")
    with pytest.raises(CommandError, match="drape first"):
        gof.send("skirt", "any", tmp_path, client=client)
    assert (tmp_path / "gof.json").read_text(encoding="utf-8") == before


def test_default_folder_comes_from_the_environment_variable(monkeypatch, clean_env, tmp_path):
    monkeypatch.setenv("FORMSCULPT_GOF", str(tmp_path / "shared"))
    assert gof.default_folder() == tmp_path / "shared"


def test_default_folder_without_the_variable_is_formsculpt_gof_in_the_home(monkeypatch, clean_env, tmp_path):
    monkeypatch.setenv("HOME", str(tmp_path))
    monkeypatch.setenv("USERPROFILE", str(tmp_path))
    assert gof.default_folder() == tmp_path / "Formsculpt" / "GoF"


def test_send_without_a_folder_writes_into_the_default_folder(client, engine, monkeypatch, clean_env, tmp_path):
    writing_engine(engine)
    monkeypatch.setenv("FORMSCULPT_GOF", str(tmp_path / "drop"))
    out = gof.send("shirt", "any", client=client)
    assert out["folder"] == str(tmp_path / "drop")
    assert (tmp_path / "drop" / "gof.json").is_file()


def test_send_result_carries_the_entry_and_the_export_reply(client, engine, tmp_path):
    writing_engine(engine, vertices=9, triangles=4)
    out = gof.send("shirt", "any", tmp_path, client=client)
    assert out["written"] == {"vertices": 9, "triangles": 4}
    assert out["entry"] == manifest(tmp_path)["objects"][0]
