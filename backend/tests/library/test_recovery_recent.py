import json
import os
from pathlib import Path

import pytest

from app.services import autosave, recent, recovery


@pytest.fixture
def data(tmp_path, monkeypatch):
    folder = tmp_path / "data"
    monkeypatch.setenv("FORMFABRIC_DATA_DIR", str(folder))
    return folder


class Journal:
    def __init__(self, patch):
        self.patch = patch
        self.handles = []

    def start(self):
        info = recovery.start()
        if recovery._owner is not None and recovery._owner not in self.handles:
            self.handles.append(recovery._owner)
        return info

    def crash(self):
        if recovery._owner is not None:
            recovery._owner.close()
        self.patch.setattr(recovery, "_owner", None)
        self.patch.setattr(recovery, "_closed", True)
        self.patch.setattr(recovery, "_done", 0)
        self.patch.setattr(recovery, "_undone", 0)
        self.patch.setattr(recovery, "_document", None)

    def quit(self):
        recovery.close()
        self.crash()


@pytest.fixture
def journal(data, monkeypatch):
    fresh = dict(_owner=None, _closed=True, _off_reason=recovery.JOURNAL_ERROR, _count=0, _log_bytes=0, _done=0, _undone=0, _truncated=False, _document=None)
    for name, value in fresh.items():
        monkeypatch.setattr(recovery, name, value)
    session = Journal(monkeypatch)
    yield session
    for handle in session.handles:
        handle.close()


class Recorder:
    def __init__(self, fail=()):
        self.calls = []
        self.fail = set(fail)

    def __call__(self, name, args):
        self.calls.append((name, args))
        if name in self.fail:
            return {"ok": False, "error": {"message": "no such panel"}}
        return {"ok": True}


def work(*commands):
    for name, args in commands:
        recovery.record(name, args, None)


def test_first_start_on_a_fresh_data_folder_offers_nothing(journal):
    assert journal.start() is None
    assert recovery.protection() == {"autosave_active": True, "autosave_off_reason": None}


def test_clean_quit_leaves_nothing_to_recover(journal):
    journal.start()
    work(("panel.add", {"w": 10}))
    journal.quit()
    assert journal.start() is None


def test_crash_with_logged_commands_is_offered_on_next_start(journal):
    journal.start()
    work(("panel.add", {"w": 10}), ("panel.move", {"dx": 2}), ("sew.add", {"a": 1, "b": 2}))
    journal.crash()
    info = journal.start()
    assert info["pending"] is True
    assert info["commands"] == 3
    assert info["base"] == "empty"
    assert info["autosave_time"] is None


def test_crash_with_no_work_at_all_offers_nothing(journal):
    journal.start()
    journal.crash()
    assert journal.start() is None


def test_recovery_files_on_disk_hold_the_commands_in_order_one_json_line_each(journal, data):
    journal.start()
    work(("panel.add", {"w": 10}), ("panel.move", {"dx": 2}))
    journal.crash()
    journal.start()
    lines = (data / "autosave" / "recovery" / "journal.jsonl").read_text(encoding="utf-8").splitlines()
    rows = [json.loads(line) for line in lines]
    assert [(r["n"], r["id"], r["args"]) for r in rows] == [(1, "panel.add", {"w": 10}), (2, "panel.move", {"dx": 2})]


def test_save_and_export_commands_are_not_written_to_the_journal(journal):
    journal.start()
    work(("file.save", {"path": "/x"}), ("panel.add", {}), ("fabric.export", {}), ("link.push", {}))
    journal.crash()
    assert journal.start()["commands"] == 1


def test_unhandled_recovery_is_offered_again_after_a_later_clean_session(journal):
    journal.start()
    work(("panel.add", {"w": 10}), ("panel.move", {"dx": 2}))
    journal.crash()
    journal.start()
    journal.quit()
    again = journal.start()
    assert again["pending"] is True
    assert again["commands"] == 2


def test_autosave_checkpoint_becomes_the_base_and_only_later_commands_are_counted(journal):
    journal.start()
    work(("panel.add", {"w": 10}), ("panel.move", {"dx": 2}))
    Path(autosave.path()).write_bytes(b"autosaved project bytes")
    recovery.checkpoint()
    work(("sew.add", {"a": 1}))
    journal.crash()
    info = journal.start()
    assert info["base"] == "autosave"
    assert info["commands"] == 1
    assert info["autosave_time"] is not None


def test_autosave_file_is_copied_into_the_recovery_folder_byte_for_byte(journal, data):
    journal.start()
    Path(autosave.path()).write_bytes(b"autosaved project bytes")
    recovery.checkpoint()
    journal.crash()
    journal.start()
    assert (data / "autosave" / "recovery" / "autosave.ffbrc").read_bytes() == b"autosaved project bytes"


def test_later_autosave_does_not_change_the_copy_kept_for_recovery(journal, data):
    journal.start()
    Path(autosave.path()).write_bytes(b"before crash")
    recovery.checkpoint()
    journal.crash()
    journal.start()
    Path(autosave.path()).write_bytes(b"new session")
    recovery.checkpoint()
    assert (data / "autosave" / "recovery" / "autosave.ffbrc").read_bytes() == b"before crash"
    assert recovery.pending_info()["pending"] is True


def test_autosave_path_is_the_file_the_journal_uses_inside_the_data_folder(data):
    assert Path(autosave.path()) == data / "autosave" / "autosave.ffbrc"
    assert Path(autosave.path()) == recovery.autosave_file()


def test_autosave_does_not_save_while_the_journal_is_off(journal):
    assert autosave.save_now(force=True) is False
    assert not Path(autosave.path()).exists()


def test_discard_removes_the_offer_and_says_so_once(journal, data):
    journal.start()
    work(("panel.add", {}))
    journal.crash()
    journal.start()
    assert recovery.discard() is True
    assert recovery.pending_info() is None
    assert not (data / "autosave" / "recovery").exists()
    assert recovery.discard() is False


def test_restore_repeats_logged_commands_in_order_on_a_new_document(journal):
    journal.start()
    work(("panel.add", {"w": 10}), ("panel.move", {"dx": 2}), ("sew.add", {"a": 1, "b": 2}))
    journal.crash()
    journal.start()
    seen = Recorder()
    result = recovery.restore(seen)
    assert seen.calls == [("file.new", {}), ("panel.add", {"w": 10}), ("panel.move", {"dx": 2}), ("sew.add", {"a": 1, "b": 2})]
    assert (result["restored"], result["logged"], result["replayed"], result["failed_count"], result["skipped_count"]) == (True, 3, 3, 0, 0)
    assert "3 of 3 logged commands" in result["summary"]


def test_restore_opens_the_kept_autosave_before_repeating_commands(journal, data):
    journal.start()
    Path(autosave.path()).write_bytes(b"base")
    recovery.checkpoint()
    work(("panel.add", {"w": 10}))
    journal.crash()
    journal.start()
    seen = Recorder()
    result = recovery.restore(seen)
    assert seen.calls == [("file.open", {"path": str(data / "autosave" / "recovery" / "autosave.ffbrc")}), ("panel.add", {"w": 10})]
    assert result["base"] == "autosave"


def test_restore_is_offered_only_once(journal):
    journal.start()
    work(("panel.add", {}))
    journal.crash()
    journal.start()
    recovery.restore(Recorder())
    assert recovery.pending_info() is None
    with pytest.raises(LookupError, match="nothing to recover"):
        recovery.restore(Recorder())


def test_restore_reports_commands_that_failed_and_goes_on(journal):
    journal.start()
    work(("panel.add", {}), ("panel.move", {"dx": 1}), ("sew.add", {}))
    journal.crash()
    journal.start()
    seen = Recorder(fail={"panel.move"})
    result = recovery.restore(seen)
    assert [name for name, _ in seen.calls] == ["file.new", "panel.add", "panel.move", "sew.add"]
    assert result["replayed"] == 2
    assert result["failed"] == [{"id": "panel.move", "error": "no such panel"}]


def test_restore_stops_with_a_message_when_the_autosave_will_not_open(journal):
    journal.start()
    Path(autosave.path()).write_bytes(b"base")
    recovery.checkpoint()
    work(("panel.add", {}))
    journal.crash()
    journal.start()
    with pytest.raises(RuntimeError, match="the autosave could not be opened"):
        recovery.restore(Recorder(fail={"file.open"}))
    assert recovery.pending_info()["pending"] is True


def test_command_that_wrote_a_file_is_left_out_of_the_replay_with_a_reason(journal):
    journal.start()
    work(("panel.add", {}), ("exchange.export_dxf", {"path": "/x.dxf"}))
    journal.crash()
    journal.start()
    seen = Recorder()
    result = recovery.restore(seen)
    assert [name for name, _ in seen.calls] == ["file.new", "panel.add"]
    assert result["skipped"] == [{"id": "exchange.export_dxf", "reason": "it wrote or read a file and is not repeated"}]
    assert result["replayed"] == 1


def test_command_with_oversized_argument_is_reported_and_not_repeated(journal):
    journal.start()
    work(("print.set", {"image": "x" * 9000, "panel": 1}), ("panel.add", {}))
    journal.crash()
    journal.start()
    seen = Recorder()
    result = recovery.restore(seen)
    assert [name for name, _ in seen.calls] == ["file.new", "panel.add"]
    assert result["failed"] == [{"id": "print.set", "error": "its arguments were too large to keep in the journal"}]


def test_undo_then_redo_in_the_journal_are_repeated_as_undo_then_redo(journal):
    journal.start()
    work(("panel.add", {}), ("history.undo", {}), ("history.redo", {}))
    journal.crash()
    journal.start()
    seen = Recorder()
    result = recovery.restore(seen)
    assert [name for name, _ in seen.calls] == ["file.new", "panel.add", "history.undo", "history.redo"]
    assert result["replayed"] == 3


def test_undo_with_nothing_to_undo_is_not_logged(journal):
    journal.start()
    work(("history.undo", {}), ("panel.add", {}))
    journal.crash()
    assert journal.start()["commands"] == 1


def test_unreadable_journal_line_is_reported_and_the_rest_is_still_repeated(journal, data):
    journal.start()
    work(("panel.add", {}))
    with (data / "autosave" / "journal.jsonl").open("a", encoding="utf-8") as handle:
        handle.write('{"n":2,"id":"panel.mo\n')
    work(("sew.add", {}))
    journal.crash()
    journal.start()
    seen = Recorder()
    result = recovery.restore(seen)
    assert [name for name, _ in seen.calls] == ["file.new", "panel.add", "sew.add"]
    assert result["failed_count"] == 1
    assert result["failed"][0]["error"] == "this line of the journal could not be read"


def test_restore_names_the_document_that_was_open_when_it_still_exists(journal, tmp_path):
    saved = tmp_path / "Coat.ffbrc"
    saved.write_bytes(b"x")
    journal.start()
    recovery.note_document(str(saved))
    work(("panel.add", {}))
    journal.crash()
    info = journal.start()
    assert info["document"] == str(saved)
    result = recovery.restore(Recorder())
    assert (result["path"], result["name"]) == (str(saved), "Coat.ffbrc")


def test_restore_names_no_document_when_that_file_is_gone(journal, tmp_path):
    saved = tmp_path / "Coat.ffbrc"
    saved.write_bytes(b"x")
    journal.start()
    recovery.note_document(str(saved))
    work(("panel.add", {}))
    journal.crash()
    journal.start()
    saved.unlink()
    result = recovery.restore(Recorder())
    assert (result["path"], result["name"]) == (None, None)


def test_second_crash_keeps_the_earlier_recovery_aside_and_offers_the_newer_one(journal, data):
    journal.start()
    work(("panel.add", {"first": True}))
    journal.crash()
    journal.start()
    work(("sew.add", {}), ("sew.add", {}))
    journal.crash()
    info = journal.start()
    assert info["commands"] == 2
    kept = (data / "autosave" / "recovery.1" / "journal.jsonl").read_text(encoding="utf-8").splitlines()
    assert [json.loads(line)["args"] for line in kept] == [{"first": True}]


@pytest.mark.skipif(os.name != "posix", reason="the lock of a second service is taken with flock here")
def test_second_service_on_the_same_data_folder_does_not_journal_or_discard(journal, data):
    import fcntl

    journal.start()
    work(("panel.add", {}))
    journal.crash()
    journal.start()
    journal.crash()
    other = (data / "autosave" / "backend.lock").open("a+b")
    try:
        fcntl.flock(other, fcntl.LOCK_EX | fcntl.LOCK_NB)
        journal.start()
        assert recovery.protection() == {"autosave_active": False, "autosave_off_reason": "second_service"}
        work(("sew.add", {}))
        assert (data / "autosave" / "journal.jsonl").read_text(encoding="utf-8") == ""
        assert recovery.discard() is False
        assert (data / "autosave" / "recovery" / "info.json").is_file()
        with pytest.raises(RuntimeError, match="another FormFabric service"):
            recovery.restore(Recorder())
    finally:
        other.close()


def test_recovery_route_reports_the_offer_after_a_crash(client, journal):
    journal.start()
    work(("panel.add", {}), ("panel.move", {}))
    journal.crash()
    journal.start()
    reply = client.get("/api/recovery").json()
    assert (reply["pending"], reply["commands"], reply["base"], reply["autosave_active"]) == (True, 2, "empty", True)


def test_recovery_route_reports_nothing_pending_on_a_fresh_folder(client, journal):
    journal.start()
    reply = client.get("/api/recovery").json()
    assert reply == {"pending": False, "autosave_time": None, "commands": 0, "autosave_active": True, "autosave_off_reason": None}


def test_discard_route_drops_the_offer_and_then_reports_nothing_to_discard(client, journal):
    journal.start()
    work(("panel.add", {}))
    journal.crash()
    journal.start()
    assert client.post("/api/recovery/discard").json() == {"discarded": True}
    assert client.get("/api/recovery").json()["pending"] is False
    assert client.post("/api/recovery/discard").json() == {"discarded": False}


def test_discard_route_without_window_key_is_refused_and_keeps_the_offer(bare, journal):
    journal.start()
    work(("panel.add", {}))
    journal.crash()
    journal.start()
    assert bare.post("/api/recovery/discard").status_code in (401, 403)
    assert recovery.pending_info()["pending"] is True


def files(tmp_path, count):
    made = []
    for i in range(count):
        path = tmp_path / f"file{i:02d}.ffbrc"
        path.write_bytes(b"x")
        made.append(path.resolve())
    return made


def test_recent_list_is_newest_first(client, data, tmp_path):
    a, b, c = files(tmp_path, 3)
    for path in (a, b, c):
        recent.note(str(path))
    assert [f["path"] for f in client.get("/api/recent").json()["files"]] == [str(c), str(b), str(a)]


def test_opening_a_file_again_moves_it_to_the_front_without_a_second_row(client, data, tmp_path):
    a, b, c = files(tmp_path, 3)
    for path in (a, b, c, a):
        recent.note(str(path))
    assert [f["name"] for f in client.get("/api/recent").json()["files"]] == ["file00.ffbrc", "file02.ffbrc", "file01.ffbrc"]


def test_recent_list_keeps_only_the_newest_twenty(client, data, tmp_path):
    made = files(tmp_path, 25)
    for path in made:
        recent.note(str(path))
    listed = [f["path"] for f in client.get("/api/recent").json()["files"]]
    assert listed == [str(p) for p in reversed(made[5:])]
    assert len(json.loads((data / "recent.json").read_text())) == 20


def test_recent_row_carries_file_name_and_file_time_in_utc(client, data, tmp_path):
    (a,) = files(tmp_path, 1)
    os.utime(a, (1_000_000_000, 1_000_000_000))
    recent.note(str(a))
    assert client.get("/api/recent").json()["files"] == [{"path": str(a), "name": "file00.ffbrc", "modified": "2001-09-09T01:46:40+00:00"}]


def test_deleted_file_drops_out_of_the_recent_list(client, data, tmp_path):
    a, b = files(tmp_path, 2)
    recent.note(str(a))
    recent.note(str(b))
    b.unlink()
    assert [f["path"] for f in client.get("/api/recent").json()["files"]] == [str(a)]
    assert [e["path"] for e in recent.entries()] == [str(a)]


def test_relative_path_is_stored_as_the_full_path(client, data, tmp_path, monkeypatch):
    (a,) = files(tmp_path, 1)
    monkeypatch.chdir(tmp_path)
    recent.note("file00.ffbrc")
    assert [f["path"] for f in client.get("/api/recent").json()["files"]] == [str(a)]


def test_removing_one_recent_file_keeps_the_others_and_the_file_itself(client, data, tmp_path):
    a, b, c = files(tmp_path, 3)
    for path in (a, b, c):
        recent.note(str(path))
    assert client.delete("/api/recent", params={"path": str(b)}).json() == {"ok": True, "removed": 1}
    assert [f["path"] for f in client.get("/api/recent").json()["files"]] == [str(c), str(a)]
    assert b.read_bytes() == b"x"


def test_removing_a_path_that_is_not_in_the_list_removes_nothing(client, data, tmp_path):
    a, b = files(tmp_path, 2)
    recent.note(str(a))
    assert client.delete("/api/recent", params={"path": str(b)}).json() == {"ok": True, "removed": 0}
    assert len(client.get("/api/recent").json()["files"]) == 1


def test_clearing_the_recent_list_reports_the_count_and_leaves_it_empty(client, data, tmp_path):
    made = files(tmp_path, 4)
    for path in made:
        recent.note(str(path))
    assert client.delete("/api/recent").json() == {"ok": True, "removed": 4}
    assert client.get("/api/recent").json() == {"files": []}
    assert all(p.is_file() for p in made)


@pytest.mark.parametrize("content", ["not json", "{}", '[1, "a", {"path": 5}]'])
def test_damaged_recent_store_reads_as_an_empty_list(client, data, content):
    data.mkdir(parents=True, exist_ok=True)
    (data / "recent.json").write_text(content, encoding="utf-8")
    assert client.get("/api/recent").json() == {"files": []}


def test_recent_list_survives_being_read_back_from_disk_by_a_plain_json_reader(data, tmp_path):
    a, b = files(tmp_path, 2)
    recent.note(str(a))
    recent.note(str(b))
    stored = json.loads((data / "recent.json").read_text(encoding="utf-8"))
    assert [row["path"] for row in stored] == [str(b), str(a)]
