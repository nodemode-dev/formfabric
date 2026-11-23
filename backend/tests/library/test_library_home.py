import io
import json
import os
import zipfile
from pathlib import Path

import pytest

from app.services import library, recent

SAMPLES = Path(__file__).resolve().parents[3] / "assets" / "samples"
PNG = b"\x89PNG\r\n\x1a\n" + bytes(range(40))


@pytest.fixture
def data(tmp_path, monkeypatch):
    folder = tmp_path / "data"
    monkeypatch.setenv("FORMFABRIC_DATA_DIR", str(folder))
    monkeypatch.delenv("FFB_TEMPLATES", raising=False)
    monkeypatch.setattr(library, "_panels", {})
    return folder


def project_bytes(panels=3):
    buffer = io.BytesIO()
    with zipfile.ZipFile(buffer, "w") as archive:
        archive.writestr("manifest.json", json.dumps({"format": "ffbrc"}))
        archive.writestr("garment.json", json.dumps({"panels": [{"n": i} for i in range(panels)]}))
    return buffer.getvalue()


def make(client, template="skirt", title=None):
    body = {"template": template}
    if title is not None:
        body["title"] = title
    reply = client.post("/api/garments", json=body)
    assert reply.status_code == 200, reply.text
    return reply.json()


def listed(client):
    return client.get("/api/garments").json()


def names(client):
    return sorted(g["name"] for g in listed(client)["garments"])


def all_files(folder):
    return sorted(str(p.relative_to(folder)) for p in folder.rglob("*") if p.is_file())


def test_empty_library_lists_no_garments_and_points_at_garments_folder_in_data_dir(client, data):
    reply = listed(client)
    assert reply["garments"] == []
    assert Path(reply["library"]) == data / "Garments"
    assert Path(reply["untitled"]) == data / "Garments" / "Untitled.ffbrc"


def test_garment_made_from_template_is_a_byte_copy_of_the_sample_on_disk(client, data):
    item = make(client, "skirt")
    saved = Path(item["path"])
    assert saved.parent == data / "Garments"
    assert saved.read_bytes() == (SAMPLES / "skirt.ffbrc").read_bytes()
    assert item["bytes"] == (SAMPLES / "skirt.ffbrc").stat().st_size


def test_new_garment_shows_in_the_list_with_its_title(client, data):
    make(client, "dress", "Summer dress")
    garments = listed(client)["garments"]
    assert [(g["title"], g["name"], g["library"]) for g in garments] == [("Summer dress", "Summer dress.ffbrc", True)]


def test_second_garment_with_same_title_gets_number_two_and_first_is_kept(client, data):
    first = make(client, "skirt", "Twin")
    make(client, "dress", "Twin")
    assert names(client) == ["Twin 2.ffbrc", "Twin.ffbrc"]
    assert Path(first["path"]).read_bytes() == (SAMPLES / "skirt.ffbrc").read_bytes()


def test_panel_count_in_list_matches_garment_json_read_with_plain_zip_reader(client, data):
    item = make(client, "tshirt-male")
    with zipfile.ZipFile(SAMPLES / "tshirt-male.ffbrc") as archive:
        expected = len(json.loads(archive.read("garment.json"))["panels"])
    assert item["panels"] == expected
    assert listed(client)["garments"][0]["panels"] == expected


@pytest.mark.parametrize("title", ["../../escape", "..\\..\\escape", "/abs/escape", "a/../../escape"])
def test_title_with_path_parts_never_writes_outside_garments_folder(client, data, tmp_path, title):
    item = make(client, "skirt", title)
    saved = Path(item["path"]).resolve()
    assert saved.parent == (data / "Garments").resolve()
    outside = [p for p in tmp_path.rglob("*.ffbrc") if p.resolve().parent != (data / "Garments").resolve()]
    assert outside == []


def test_title_that_is_a_reserved_device_name_is_not_used_as_file_name(client, data):
    item = make(client, "skirt", "CON")
    assert Path(item["path"]).stem.lower() != "con"
    assert Path(item["path"]).is_file()


def test_unknown_template_is_refused_and_nothing_is_written(client, data):
    reply = client.post("/api/garments", json={"template": "trousers"})
    assert reply.status_code == 404
    assert reply.json()["detail"] == "No such template."
    assert listed(client)["garments"] == []


def test_template_id_that_is_a_path_is_refused(client, data):
    reply = client.post("/api/garments", json={"template": "../samples/skirt"})
    assert reply.status_code == 404
    assert all_files(data / "Garments") == []


def test_missing_template_is_refused_with_hint_to_new_project(client, data):
    reply = client.post("/api/garments", json={})
    assert reply.status_code == 400
    assert "template is needed" in reply.json()["detail"]


def test_title_that_is_not_text_is_refused(client, data):
    reply = client.post("/api/garments", json={"template": "skirt", "title": 7})
    assert reply.status_code == 400
    assert reply.json()["detail"] == "title must be text"
    assert all_files(data / "Garments") == []


def test_create_without_window_key_is_refused_and_writes_nothing(bare, client, data):
    reply = bare.post("/api/garments", json={"template": "skirt"})
    assert reply.status_code in (401, 403)
    assert listed(client)["garments"] == []


def test_readme_four_sample_garments_are_offered_as_templates(client, data):
    reply = client.get("/api/garment-templates").json()
    assert sorted(t["id"] for t in reply["templates"]) == ["dress", "skirt", "tshirt-female", "tshirt-male"]
    assert reply["categories"] == ["Dresses", "Skirts", "Tops"]


def test_template_preview_is_the_sample_picture(client, data):
    reply = client.get("/api/garment-templates/dress/thumb")
    assert reply.headers["content-type"] == "image/webp"
    assert reply.content == (SAMPLES / "dress.webp").read_bytes()


def test_duplicate_makes_a_copy_named_copy_with_same_bytes(client, data):
    item = make(client, "skirt", "Base")
    copy = client.post(f"/api/garments/{item['id']}/duplicate").json()
    assert copy["name"] == "Base copy.ffbrc"
    assert Path(copy["path"]).read_bytes() == Path(item["path"]).read_bytes()
    assert copy["id"] != item["id"]
    assert names(client) == ["Base copy.ffbrc", "Base.ffbrc"]


def test_second_duplicate_gets_copy_two(client, data):
    item = make(client, "skirt", "Base")
    client.post(f"/api/garments/{item['id']}/duplicate")
    again = client.post(f"/api/garments/{item['id']}/duplicate").json()
    assert again["name"] == "Base copy 2.ffbrc"
    assert len(names(client)) == 3


def test_duplicate_of_unknown_garment_is_refused(client, data):
    reply = client.post("/api/garments/g_00000000000000000000/duplicate")
    assert reply.status_code == 404
    assert reply.json()["detail"] == "That garment is no longer in the list."


def test_garment_id_that_is_not_an_id_is_refused(client, data):
    reply = client.delete("/api/garments/..%2F..%2Fhome")
    assert reply.status_code == 404
    reply = client.delete("/api/garments/not-an-id")
    assert reply.status_code == 404
    assert reply.json()["detail"] == "No such garment."


def test_removed_garment_leaves_the_list_and_waits_in_trash_with_same_bytes(client, data):
    item = make(client, "dress", "Old")
    original = Path(item["path"]).read_bytes()
    reply = client.delete(f"/api/garments/{item['id']}").json()
    assert reply["ok"] is True
    assert listed(client)["garments"] == []
    assert not Path(item["path"]).exists()
    trashed = client.get("/api/garments/trash").json()["garments"]
    assert [(t["entry"], t["title"], t["id"]) for t in trashed] == [(reply["entry"], "Old", item["id"])]
    assert (data / "Garments" / ".trash" / reply["entry"] / "Old.ffbrc").read_bytes() == original


def test_restore_from_trash_brings_back_same_name_and_bytes(client, data):
    item = make(client, "dress", "Back")
    original = Path(item["path"]).read_bytes()
    entry = client.delete(f"/api/garments/{item['id']}").json()["entry"]
    restored = client.post(f"/api/garments/trash/{entry}/restore").json()
    assert restored["name"] == "Back.ffbrc"
    assert restored["id"] == item["id"]
    assert Path(restored["path"]).read_bytes() == original
    assert client.get("/api/garments/trash").json()["garments"] == []
    assert names(client) == ["Back.ffbrc"]


def test_restore_when_name_is_taken_keeps_both_files(client, data):
    item = make(client, "dress", "Clash")
    entry = client.delete(f"/api/garments/{item['id']}").json()["entry"]
    make(client, "skirt", "Clash")
    restored = client.post(f"/api/garments/trash/{entry}/restore").json()
    assert restored["name"] == "Clash (restored).ffbrc"
    assert Path(restored["path"]).read_bytes() == (SAMPLES / "dress.ffbrc").read_bytes()
    assert (data / "Garments" / "Clash.ffbrc").read_bytes() == (SAMPLES / "skirt.ffbrc").read_bytes()


def test_purged_garment_is_gone_for_good(client, data):
    item = make(client, "skirt", "Gone")
    entry = client.delete(f"/api/garments/{item['id']}").json()["entry"]
    assert client.delete(f"/api/garments/trash/{entry}").json() == {"ok": True}
    assert client.get("/api/garments/trash").json()["garments"] == []
    again = client.post(f"/api/garments/trash/{entry}/restore")
    assert again.status_code == 404
    assert again.json()["detail"] == "That garment is no longer in the trash."
    assert all_files(data / "Garments") == []


def test_empty_trash_reports_how_many_were_removed(client, data):
    for title in ("One", "Two"):
        client.delete(f"/api/garments/{make(client, 'skirt', title)['id']}")
    kept = make(client, "skirt", "Kept")
    assert client.delete("/api/garments/trash").json() == {"ok": True, "removed": 2}
    assert client.get("/api/garments/trash").json()["garments"] == []
    assert Path(kept["path"]).is_file()


def test_removing_same_garment_twice_keeps_two_trash_entries(client, data):
    first = make(client, "skirt", "Same")
    one = client.delete(f"/api/garments/{first['id']}").json()["entry"]
    second = make(client, "dress", "Same")
    two = client.delete(f"/api/garments/{second['id']}").json()["entry"]
    assert one != two
    assert sorted(t["entry"] for t in client.get("/api/garments/trash").json()["garments"]) == sorted([one, two])


@pytest.mark.parametrize("entry", ["..", "../..", "../../Garments", "g_00000000000000000000-1/..", "/etc", ""])
def test_trash_entry_names_that_are_paths_are_refused_and_delete_nothing(data, entry):
    library.library_dir()
    victim = data / "victim.txt"
    victim.write_text("keep")
    with pytest.raises(LookupError):
        library.purge(entry)
    with pytest.raises(LookupError):
        library.restore(entry)
    assert victim.read_text() == "keep"
    assert (data / "Garments").is_dir()


def test_well_formed_but_missing_trash_entry_is_refused_over_http(client, data):
    reply = client.delete("/api/garments/trash/g_00000000000000000000-5")
    assert reply.status_code == 404
    assert reply.json()["detail"] == "That garment is no longer in the trash."


@pytest.mark.skipif(os.name != "posix", reason="symbolic links need no extra rights on POSIX only")
def test_trash_entry_that_is_a_link_to_an_outside_folder_is_never_purged(client, data, tmp_path):
    outside = tmp_path / "outside"
    outside.mkdir()
    (outside / "precious.ffbrc").write_bytes(project_bytes())
    name = "g_00000000000000000000-9"
    library.trash_dir()
    (data / "Garments" / ".trash" / name).symlink_to(outside, target_is_directory=True)
    assert client.delete(f"/api/garments/trash/{name}").status_code == 404
    assert client.post(f"/api/garments/trash/{name}/restore").status_code == 404
    client.delete("/api/garments/trash")
    assert (outside / "precious.ffbrc").read_bytes() == project_bytes()


@pytest.mark.skipif(os.name != "posix", reason="symbolic links need no extra rights on POSIX only")
def test_link_inside_garments_folder_to_an_outside_file_is_not_listed(client, data, tmp_path):
    outside = tmp_path / "elsewhere.ffbrc"
    outside.write_bytes(project_bytes())
    library.library_dir()
    (data / "Garments" / "Linked.ffbrc").symlink_to(outside)
    assert listed(client)["garments"] == []


def test_hidden_files_and_other_file_types_in_garments_folder_are_not_listed(client, data):
    folder = library.library_dir()
    (folder / ".hidden.ffbrc").write_bytes(project_bytes())
    (folder / "notes.txt").write_text("hello")
    (folder / "Real.ffbrc").write_bytes(project_bytes(2))
    garments = listed(client)["garments"]
    assert [g["name"] for g in garments] == ["Real.ffbrc"]
    assert garments[0]["panels"] == 2


def test_file_that_is_not_a_zip_is_listed_with_unknown_panel_count(client, data):
    (library.library_dir() / "Broken.ffbrc").write_bytes(b"not a zip")
    garments = listed(client)["garments"]
    assert [(g["name"], g["panels"], g["bytes"]) for g in garments] == [("Broken.ffbrc", None, 9)]


def test_garments_are_listed_newest_first_with_file_time_in_utc(client, data):
    folder = library.library_dir()
    for name, stamp in (("Old.ffbrc", 1_000_000_000), ("New.ffbrc", 2_000_000_000), ("Mid.ffbrc", 1_500_000_000)):
        (folder / name).write_bytes(project_bytes())
        os.utime(folder / name, (stamp, stamp))
    garments = listed(client)["garments"]
    assert [g["name"] for g in garments] == ["New.ffbrc", "Mid.ffbrc", "Old.ffbrc"]
    assert garments[0]["modified"] == "2033-05-18T03:33:20+00:00"
    assert garments[2]["modified"] == "2001-09-09T01:46:40+00:00"


def outside_garment(tmp_path):
    outside = tmp_path / "desk" / "Mine.ffbrc"
    outside.parent.mkdir()
    outside.write_bytes(project_bytes(4))
    recent.note(str(outside))
    return outside


def test_recent_file_outside_the_library_is_listed_but_marked_as_outside(client, data, tmp_path):
    outside = outside_garment(tmp_path)
    garments = listed(client)["garments"]
    assert [(g["name"], g["library"], g["panels"]) for g in garments] == [("Mine.ffbrc", False, 4)]
    assert Path(garments[0]["path"]) == outside.resolve()


def test_readme_file_outside_garments_folder_is_never_moved_or_deleted(client, data, tmp_path):
    outside = outside_garment(tmp_path)
    gid = listed(client)["garments"][0]["id"]
    reply = client.delete(f"/api/garments/{gid}")
    assert reply.status_code == 400
    assert "outside the library" in reply.json()["detail"]
    assert outside.read_bytes() == project_bytes(4)
    assert client.get("/api/garments/trash").json()["garments"] == []
    assert [g["id"] for g in listed(client)["garments"]] == [gid]


def test_forgetting_an_outside_garment_drops_it_from_the_list_but_keeps_the_file(client, data, tmp_path):
    outside = outside_garment(tmp_path)
    gid = listed(client)["garments"][0]["id"]
    assert client.post(f"/api/garments/{gid}/forget").json() == {"ok": True}
    assert listed(client)["garments"] == []
    assert outside.read_bytes() == project_bytes(4)


def test_duplicate_of_outside_garment_lands_in_library_and_leaves_source_alone(client, data, tmp_path):
    outside = outside_garment(tmp_path)
    gid = listed(client)["garments"][0]["id"]
    copy = client.post(f"/api/garments/{gid}/duplicate").json()
    assert Path(copy["path"]).parent == data / "Garments"
    assert Path(copy["path"]).read_bytes() == outside.read_bytes()
    assert copy["library"] is True


def test_pinned_garments_are_kept_in_order_and_read_back(client, data):
    a, b = make(client, "skirt", "A"), make(client, "dress", "B")
    reply = client.post("/api/garments/settings", json={"pinned": [b["id"], a["id"], b["id"], "junk", 5]})
    assert reply.json()["pinned"] == [b["id"], a["id"]]
    again = listed(client)
    assert again["pinned"] == [b["id"], a["id"]]
    assert again["recentHidden"] == []
    assert json.loads((data / "home.json").read_text())["pinned"] == [b["id"], a["id"]]


def test_pinned_that_is_not_a_list_is_refused_and_old_pins_stay(client, data):
    a = make(client, "skirt", "A")
    client.post("/api/garments/settings", json={"pinned": [a["id"]]})
    reply = client.post("/api/garments/settings", json={"pinned": "all"})
    assert reply.status_code == 400
    assert reply.json()["detail"] == "pinned must be a list of garment ids"
    assert listed(client)["pinned"] == [a["id"]]


def test_removing_a_pinned_garment_unpins_it(client, data):
    a, b = make(client, "skirt", "A"), make(client, "dress", "B")
    client.post("/api/garments/settings", json={"pinned": [a["id"], b["id"]], "recentHidden": [a["id"]]})
    client.delete(f"/api/garments/{a['id']}")
    after = listed(client)
    assert after["pinned"] == [b["id"]]
    assert after["recentHidden"] == []


def test_preview_picture_round_trips_and_is_flagged_in_the_list(client, data):
    item = make(client, "skirt", "Pic")
    reply = client.put("/api/garments/thumb", params={"path": item["path"]}, content=PNG)
    assert reply.json() == {"ok": True, "id": item["id"]}
    got = client.get(f"/api/garments/{item['id']}/thumb")
    assert got.content == PNG
    assert got.headers["content-type"] == "image/png"
    assert listed(client)["garments"][0]["thumb"] is True


def test_preview_follows_garment_through_trash_and_back(client, data):
    item = make(client, "skirt", "Pic")
    client.put("/api/garments/thumb", params={"path": item["path"]}, content=PNG)
    entry = client.delete(f"/api/garments/{item['id']}").json()["entry"]
    assert client.get(f"/api/garments/{item['id']}/thumb").status_code == 404
    assert client.get("/api/garments/trash").json()["garments"][0]["thumb"] is True
    client.post(f"/api/garments/trash/{entry}/restore")
    assert client.get(f"/api/garments/{item['id']}/thumb").content == PNG


def test_preview_that_is_not_a_picture_is_refused(client, data):
    item = make(client, "skirt", "Pic")
    client.put("/api/garments/thumb", params={"path": item["path"]}, content=PNG)
    reply = client.put("/api/garments/thumb", params={"path": item["path"]}, content=b"<html>hello</html>")
    assert reply.status_code == 400
    assert reply.json()["detail"] == "A preview is a PNG, JPEG or WebP picture of at most 2 MB."
    assert client.get(f"/api/garments/{item['id']}/thumb").content == PNG


def test_preview_over_two_megabytes_is_refused(client, data):
    item = make(client, "skirt", "Pic")
    reply = client.put("/api/garments/thumb", params={"path": item["path"]}, content=PNG + bytes(2_000_000))
    assert reply.status_code == 413
    assert reply.json()["detail"] == "A preview is at most 2 MB."
    assert client.get(f"/api/garments/{item['id']}/thumb").content == (SAMPLES / "skirt.webp").read_bytes()


@pytest.mark.parametrize("path", ["relative.ffbrc", "/etc/passwd", "../../x.ffbrc"])
def test_preview_for_a_path_that_is_not_a_listed_garment_is_refused(client, data, tmp_path, path):
    reply = client.put("/api/garments/thumb", params={"path": path}, content=PNG)
    assert reply.status_code == 404
    assert all_files(data) == []


def test_preview_for_unlisted_absolute_garment_path_is_refused(client, data, tmp_path):
    stray = tmp_path / "stray.ffbrc"
    stray.write_bytes(project_bytes())
    reply = client.put("/api/garments/thumb", params={"path": str(stray)}, content=PNG)
    assert reply.status_code == 404
    assert reply.json()["detail"] == "That garment is no longer in the list."


def test_brought_in_project_is_stored_byte_for_byte_and_listed(client, data):
    payload = project_bytes(5)
    reply = client.post("/api/garments/import", params={"name": "Coat.ffbrc"}, content=payload).json()
    assert reply["kind"] == "garment"
    assert Path(reply["path"]) == data / "Garments" / "Coat.ffbrc"
    assert Path(reply["path"]).read_bytes() == payload
    assert [(g["name"], g["panels"]) for g in listed(client)["garments"]] == [("Coat.ffbrc", 5)]


def test_bringing_in_same_name_twice_keeps_both(client, data):
    client.post("/api/garments/import", params={"name": "Coat.ffbrc"}, content=project_bytes(1))
    client.post("/api/garments/import", params={"name": "Coat.ffbrc"}, content=project_bytes(2))
    assert sorted((g["name"], g["panels"]) for g in listed(client)["garments"]) == [("Coat 2.ffbrc", 2), ("Coat.ffbrc", 1)]


def test_file_that_is_not_a_project_is_refused_and_leaves_no_file_behind(client, data):
    reply = client.post("/api/garments/import", params={"name": "Fake.ffbrc"}, content=b"plain text")
    assert reply.status_code == 400
    assert reply.json()["detail"] == "That file is not a FormFabric project."
    assert all_files(data / "Garments") == []


def test_zip_without_manifest_is_refused_as_not_a_project(client, data):
    buffer = io.BytesIO()
    with zipfile.ZipFile(buffer, "w") as archive:
        archive.writestr("garment.json", "{}")
    reply = client.post("/api/garments/import", params={"name": "Half.ffbrc"}, content=buffer.getvalue())
    assert reply.status_code == 400
    assert all_files(data / "Garments") == []


def test_other_file_types_are_refused_by_name(client, data):
    reply = client.post("/api/garments/import", params={"name": "photo.png"}, content=PNG)
    assert reply.status_code == 400
    assert reply.json()["detail"] == "Only .ffbrc garment projects and .dxf patterns can be brought in here."


def test_empty_upload_is_refused_and_leaves_no_file_behind(client, data):
    reply = client.post("/api/garments/import", params={"name": "Empty.ffbrc"}, content=b"")
    assert reply.status_code == 400
    assert reply.json()["detail"] == "That file is empty."
    assert all_files(data / "Garments") == []


def test_dxf_pattern_goes_to_imports_folder_and_not_into_the_garment_list(client, data):
    payload = b"0\nSECTION\n2\nENTITIES\n0\nENDSEC\n0\nEOF\n"
    reply = client.post("/api/garments/import", params={"name": "Block.dxf"}, content=payload).json()
    assert reply == {"kind": "dxf", "path": str(data / "Imports" / "Block.dxf")}
    assert (data / "Imports" / "Block.dxf").read_bytes() == payload
    assert listed(client)["garments"] == []


@pytest.mark.parametrize("name,folder", [("../../evil.dxf", "Imports"), ("..\\..\\evil.dxf", "Imports"), ("../../evil.ffbrc", "Garments"), ("/tmp/evil.ffbrc", "Garments")])
def test_upload_name_with_path_parts_stays_inside_its_folder(client, data, tmp_path, name, folder):
    reply = client.post("/api/garments/import", params={"name": name}, content=project_bytes())
    assert reply.status_code == 200
    saved = Path(reply.json()["path"]).resolve()
    assert saved.parent == (data / folder).resolve()
    strays = [p for p in tmp_path.rglob("*evil*") if p.resolve().parent != (data / folder).resolve()]
    assert strays == []
