import json
import logging
import os
from logging.handlers import RotatingFileHandler

import pytest

from app.core import quiet_log, runtime, startup


@pytest.fixture
def access_logger(monkeypatch):
    log = logging.getLogger("uvicorn.access")
    before = list(log.filters)
    seen = []

    class Collect(logging.Handler):
        def emit(self, record):
            seen.append(record.getMessage())

    handler = Collect()
    log.addHandler(handler)
    monkeypatch.setattr(log, "level", logging.INFO)
    monkeypatch.setattr(log, "propagate", False)
    log.filters[:] = [f for f in log.filters if type(f).__name__ != "_FailuresOnly"]
    yield log, seen
    log.removeHandler(handler)
    log.filters[:] = before


def hit(log, status, path="/api/x"):
    log.info('%s - "%s %s HTTP/%s" %d', "127.0.0.1:5000", "GET", path, "1.1", status)


def test_without_quiet_logging_every_request_is_in_the_access_log(access_logger, monkeypatch):
    log, seen = access_logger
    hit(log, 200)
    hit(log, 404)
    assert len(seen) == 2


def test_quiet_logging_keeps_failures_and_drops_successes(access_logger, monkeypatch):
    log, seen = access_logger
    monkeypatch.delenv("FFB_ACCESS_LOG", raising=False)
    quiet_log.install()
    for status in (200, 204, 304, 399, 400, 403, 404, 500):
        hit(log, status)
    assert [m.rsplit(" ", 1)[-1] for m in seen] == ["400", "403", "404", "500"]


def test_installing_quiet_logging_twice_does_not_stack_filters(access_logger, monkeypatch):
    log, seen = access_logger
    monkeypatch.delenv("FFB_ACCESS_LOG", raising=False)
    quiet_log.install()
    quiet_log.install()
    assert len([f for f in log.filters if type(f).__name__ == "_FailuresOnly"]) == 1


def test_asking_for_the_full_access_log_leaves_every_request_in_it(access_logger, monkeypatch):
    log, seen = access_logger
    monkeypatch.setenv("FFB_ACCESS_LOG", "1")
    quiet_log.install()
    hit(log, 200)
    assert len(seen) == 1


def test_other_log_lines_are_not_dropped_by_quiet_logging(access_logger, monkeypatch):
    log, seen = access_logger
    monkeypatch.delenv("FFB_ACCESS_LOG", raising=False)
    quiet_log.install()
    log.info("plain note")
    assert seen == ["plain note"]


def test_the_engine_log_lives_in_a_logs_folder_of_the_data_folder(tmp_path):
    path = runtime.log_path(tmp_path / "d")
    assert path == tmp_path / "d" / "Logs" / "formfabric-engine.log"
    assert path.parent.is_dir()


@pytest.fixture
def engine_log():
    log = logging.getLogger("formfabric")
    before_handlers, before_level = list(log.handlers), log.level
    yield log
    for h in list(log.handlers):
        if h not in before_handlers:
            log.removeHandler(h)
            h.close()
    log.setLevel(before_level)


def test_starting_the_log_writes_a_line_that_names_the_process_and_data_folder(tmp_path, engine_log):
    path = runtime.start_logging(tmp_path)
    for h in engine_log.handlers:
        h.flush()
    text = path.read_text(encoding="utf-8")
    assert "INFO formfabric: engine started" in text
    assert f"pid {os.getpid()}" in text and str(tmp_path) in text


def test_later_messages_to_the_formfabric_logger_reach_the_file(tmp_path, engine_log):
    path = runtime.start_logging(tmp_path)
    logging.getLogger("formfabric.commands").warning("a command failed on purpose")
    for h in engine_log.handlers:
        h.flush()
    assert "WARNING formfabric.commands: a command failed on purpose" in path.read_text(encoding="utf-8")


def test_the_log_rotates_at_one_mebibyte_and_keeps_three_old_files(tmp_path, engine_log):
    runtime.start_logging(tmp_path)
    handler = [h for h in engine_log.handlers if isinstance(h, RotatingFileHandler)][-1]
    assert (handler.maxBytes, handler.backupCount) == (1024 * 1024, 3)


def test_the_server_log_setup_sends_server_lines_to_stdout_without_colour_and_to_the_file(tmp_path):
    from uvicorn.config import LOGGING_CONFIG

    original = json.dumps(LOGGING_CONFIG, sort_keys=True, default=str)
    config = runtime.server_log_config(tmp_path)
    assert all(h["stream"] == "ext://sys.stdout" for n, h in config["handlers"].items() if n != "file")
    assert all(f.get("use_colors") is False for n, f in config["formatters"].items() if n != "file")
    assert config["handlers"]["file"]["filename"] == str(tmp_path / "Logs" / "formfabric-engine.log")
    assert "file" in config["loggers"]["uvicorn"]["handlers"] and "file" in config["loggers"]["uvicorn.access"]["handlers"]
    assert json.dumps(LOGGING_CONFIG, sort_keys=True, default=str) == original


def test_the_server_log_setup_is_a_valid_logging_configuration(tmp_path):
    import logging.config

    config = runtime.server_log_config(tmp_path)
    saved = {n: (list(logging.getLogger(n).handlers), logging.getLogger(n).level, logging.getLogger(n).propagate) for n in ("uvicorn", "uvicorn.error", "uvicorn.access")}
    try:
        logging.config.dictConfig(config)
        logging.getLogger("uvicorn.error").error("written to the file")
        for h in logging.getLogger("uvicorn").handlers:
            h.flush()
        assert "written to the file" in (tmp_path / "Logs" / "formfabric-engine.log").read_text(encoding="utf-8")
    finally:
        for n, (handlers, level, propagate) in saved.items():
            log = logging.getLogger(n)
            for h in list(log.handlers):
                if h not in handlers:
                    h.close()
            log.handlers[:] = handlers
            log.setLevel(level)
            log.propagate = propagate


def test_releasing_removes_the_lock_this_engine_holds(tmp_path):
    lock = tmp_path / startup.LOCK_NAME
    lock.write_text(json.dumps({"pid": os.getpid()}))
    runtime.release(tmp_path)
    assert not lock.exists()


def test_releasing_leaves_a_lock_held_by_another_engine(tmp_path):
    lock = tmp_path / startup.LOCK_NAME
    lock.write_text(json.dumps({"pid": os.getpid() + 100000}))
    runtime.release(tmp_path)
    assert lock.exists()


@pytest.mark.parametrize("text", ["", "{", "[1]", "{\"pid\": \"abc\"}", "null"])
def test_releasing_over_a_damaged_lock_changes_nothing_and_does_not_fail(tmp_path, text):
    lock = tmp_path / startup.LOCK_NAME
    lock.write_text(text)
    runtime.release(tmp_path)
    assert lock.read_text() == text


def test_releasing_with_no_lock_does_nothing(tmp_path):
    runtime.release(tmp_path)
    assert list(tmp_path.iterdir()) == []
