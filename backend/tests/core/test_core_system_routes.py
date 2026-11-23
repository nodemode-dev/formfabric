import platform
import sys

import pytest

from app import __version__

from .support import PNG, REFUSED, data_folder

JPEG = b"\xff\xd8\xff\xe0" + b"\x00" * 32
GIF = b"GIF89a" + b"\x00" * 32
WEBP = b"RIFF\x00\x00\x00\x00WEBP" + b"\x00" * 32


def test_health_says_ok_with_the_app_name_and_its_own_version(bare):
    assert bare.get("/api/health").json() == {"ok": True, "app": "formfabric", "version": __version__}


def test_the_version_route_and_health_and_capabilities_agree_on_the_version(bare):
    versions = {bare.get("/api/health").json()["version"], bare.get("/api/version").json()["version"], bare.get("/api/capabilities").json()["version"], bare.get("/openapi.json").json()["info"]["version"]}
    assert versions == {__version__}


def test_the_version_route_names_the_app_and_a_dotted_dock_contract(bare):
    body = bare.get("/api/version").json()
    assert body["app"] == "formfabric"
    assert [p.isdigit() for p in body["dock"].split(".")] == [True, True, True]


def test_capabilities_report_the_python_and_the_system_it_runs_on(bare):
    body = bare.get("/api/capabilities").json()
    assert body["python"] == sys.version.split()[0]
    assert body["platform"] == platform.system().lower()
    assert body["app"] == "formfabric"


def test_capabilities_say_the_native_engine_is_not_built_and_list_no_commands(bare):
    assert bare.get("/api/capabilities").json()["engine"] == {"native": False, "version": None, "commands": []}


@pytest.mark.parametrize("value,expected", [("1", True), ("0", False), ("", False)])
def test_capabilities_follow_the_desktop_setting_at_the_time_of_asking(bare, monkeypatch, value, expected):
    monkeypatch.setenv("FFB_DESKTOP", value)
    assert bare.get("/api/capabilities").json()["desktop"] is expected


def test_capabilities_say_whether_a_frontend_build_is_there(bare, tmp_path, monkeypatch):
    monkeypatch.setenv("FFB_DIST", str(tmp_path))
    assert bare.get("/api/capabilities").json()["frontend_built"] is False
    (tmp_path / "index.html").write_text("<html></html>")
    assert bare.get("/api/capabilities").json()["frontend_built"] is True


def test_the_catalogue_says_the_engine_module_is_not_built_and_how_to_build_it(bare):
    r = bare.get("/api/engine/catalogue")
    assert r.status_code == 503
    assert r.json() == {"detail": "the engine module is not built; run ./formfabric engine build"}


def test_a_dispatch_to_the_missing_engine_is_refused_with_the_same_message(client):
    r = client.post("/api/engine/dispatch", json={"name": "file.new", "args": {}})
    assert r.status_code == 503
    assert r.json()["detail"] == "the engine module is not built; run ./formfabric engine build"


def test_the_stream_route_for_the_missing_engine_is_refused_with_the_message(client):
    r = client.get("/api/engine/stream")
    assert r.status_code == 503 and "engine module is not built" in r.json()["detail"]


def test_a_dispatch_without_the_window_key_is_refused_for_the_key_first(bare):
    r = bare.post("/api/engine/dispatch", json={"name": "file.new", "args": {}})
    assert r.status_code == 403 and r.json() == REFUSED


def test_the_stream_route_without_the_window_key_is_refused(bare):
    r = bare.get("/api/engine/stream")
    assert r.status_code == 403 and r.json() == REFUSED


def test_a_dispatch_with_a_name_over_120_letters_is_refused_as_malformed(client):
    r = client.post("/api/engine/dispatch", json={"name": "x" * 121, "args": {}})
    assert r.status_code == 422


def test_a_dispatch_with_no_name_is_refused_as_malformed(client):
    assert client.post("/api/engine/dispatch", json={"args": {}}).status_code == 422


def test_the_window_key_route_does_not_reveal_a_key_when_asked_for_state(bare):
    body = bare.get("/api/window/key").json()
    assert set(body) == {"secondClaim"}


def test_the_recent_list_starts_empty_and_clearing_it_removes_nothing(client, data_folder):
    assert client.get("/api/recent").json() == {"files": []}
    assert client.delete("/api/recent").json() == {"ok": True, "removed": 0}


def test_a_remembered_file_is_listed_with_its_name_and_forgotten_on_request(client, data_folder, tmp_path):
    from app.services import recent

    shirt = tmp_path / "shirt.ffb"
    shirt.write_text("x")
    other = tmp_path / "skirt.ffb"
    other.write_text("y")
    recent.note(str(shirt))
    recent.note(str(other))
    files = client.get("/api/recent").json()["files"]
    assert [f["name"] for f in files] == ["skirt.ffb", "shirt.ffb"]
    assert files[1]["path"] == str(shirt.resolve())
    assert client.delete("/api/recent", params={"path": str(shirt)}).json() == {"ok": True, "removed": 1}
    assert [f["name"] for f in client.get("/api/recent").json()["files"]] == ["skirt.ffb"]
    assert client.delete("/api/recent").json()["ok"] is True
    assert client.get("/api/recent").json() == {"files": []}


def test_a_remembered_file_that_has_since_been_deleted_drops_off_the_list(client, data_folder, tmp_path):
    from app.services import recent

    gone = tmp_path / "gone.ffb"
    gone.write_text("x")
    recent.note(str(gone))
    gone.unlink()
    assert client.get("/api/recent").json() == {"files": []}


def test_the_recent_list_holds_at_most_twenty_files_newest_first(client, data_folder, tmp_path):
    from app.services import recent

    for n in range(23):
        f = tmp_path / f"f{n:02d}.ffb"
        f.write_text("x")
        recent.note(str(f))
    names = [f["name"] for f in client.get("/api/recent").json()["files"]]
    assert len(names) == 20
    assert names[0] == "f22.ffb" and names[-1] == "f03.ffb"


def test_noting_the_same_file_twice_lists_it_once_at_the_top(client, data_folder, tmp_path):
    from app.services import recent

    a, b = tmp_path / "a.ffb", tmp_path / "b.ffb"
    a.write_text("x")
    b.write_text("x")
    for f in (a, b, a):
        recent.note(str(f))
    assert [f["name"] for f in client.get("/api/recent").json()["files"]] == ["a.ffb", "b.ffb"]


@pytest.mark.parametrize("name,data,kind", [("a.png", PNG, "image/png"), ("a.jpg", JPEG, "image/jpeg"), ("a.JPEG", JPEG, "image/jpeg"), ("a.gif", GIF, "image/gif"), ("a.webp", WEBP, "image/webp")])
def test_an_image_file_is_served_byte_for_byte_with_its_type(client, tmp_path, name, data, kind):
    f = tmp_path / name
    f.write_bytes(data)
    r = client.get("/api/files/image", params={"path": str(f)})
    assert r.status_code == 200 and r.content == data
    assert r.headers["content-type"] == kind


def test_an_image_read_needs_the_window_key(bare, tmp_path):
    f = tmp_path / "a.png"
    f.write_bytes(PNG)
    r = bare.get("/api/files/image", params={"path": str(f)})
    assert r.status_code == 403 and r.json() == REFUSED


@pytest.mark.parametrize("path", ["relative/a.png", "a.png", "/tmp/notes.txt", "/tmp/a.svg", "/tmp/noextension"])
def test_a_path_that_is_not_an_absolute_image_path_is_refused_with_400(client, path):
    r = client.get("/api/files/image", params={"path": path})
    assert r.status_code == 400 and r.json() == {"ok": False, "error": "Not an absolute image path"}


def test_a_missing_image_is_404_with_the_stated_reason(client, tmp_path):
    r = client.get("/api/files/image", params={"path": str(tmp_path / "missing.png")})
    assert r.status_code == 404 and r.json() == {"ok": False, "error": "No such image, or larger than 20 MB"}


def test_a_file_named_png_that_is_not_a_picture_is_not_served(client, tmp_path):
    f = tmp_path / "fake.png"
    f.write_text("this is plain text, not a picture")
    assert client.get("/api/files/image", params={"path": str(f)}).status_code == 404


def test_an_image_is_typed_by_its_file_name_while_its_content_only_has_to_be_some_picture(client, tmp_path):
    f = tmp_path / "wrong.gif"
    f.write_bytes(PNG)
    r = client.get("/api/files/image", params={"path": str(f)})
    assert r.status_code == 200 and r.headers["content-type"] == "image/gif" and r.content == PNG


def test_a_link_to_an_image_is_not_followed(client, tmp_path):
    real = tmp_path / "real.png"
    real.write_bytes(PNG)
    link = tmp_path / "link.png"
    link.symlink_to(real)
    assert client.get("/api/files/image", params={"path": str(link)}).status_code == 404


def test_an_image_over_twenty_megabytes_is_not_served(client, tmp_path):
    big = tmp_path / "big.png"
    with big.open("wb") as h:
        h.write(PNG)
        h.truncate(20 * 1024 * 1024 + 1)
    assert client.get("/api/files/image", params={"path": str(big)}).status_code == 404


def test_an_image_of_exactly_twenty_megabytes_is_served(client, tmp_path):
    edge = tmp_path / "edge.png"
    with edge.open("wb") as h:
        h.write(PNG)
        h.truncate(20 * 1024 * 1024)
    r = client.get("/api/files/image", params={"path": str(edge)})
    assert r.status_code == 200 and len(r.content) == 20 * 1024 * 1024


def test_a_folder_named_like_an_image_is_not_served(client, tmp_path):
    d = tmp_path / "folder.png"
    d.mkdir()
    assert client.get("/api/files/image", params={"path": str(d)}).status_code == 404


def test_every_answer_carries_the_cross_origin_isolation_headers_so_shared_memory_works(bare, client):
    wanted = {"cross-origin-opener-policy": "same-origin", "cross-origin-embedder-policy": "require-corp", "cross-origin-resource-policy": "same-origin"}
    for r in (bare.get("/api/health"), bare.get("/api/no-such-route"), bare.post("/api/commands/agents.prefs", json={}), client.get("/api/engine/catalogue")):
        for k, v in wanted.items():
            assert r.headers[k] == v, (r.url, k)
