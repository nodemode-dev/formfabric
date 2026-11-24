import sys

import pytest

from app.core import config

from .support import run_python


@pytest.mark.parametrize("raw,expected", [("", 8280), ("9100", 9100), (" 9100 ", 9100), ("abc", 8280), ("0", 8280), ("-5", 8280), ("80.5", 8280)])
def test_the_engine_port_comes_from_the_environment_or_falls_back_to_the_documented_8280(monkeypatch, raw, expected):
    monkeypatch.setenv("FFB_PORT", raw)
    assert config.engine_port() == expected
    assert config.engine_url() == f"http://127.0.0.1:{expected}"


def test_the_engine_port_is_8280_when_nothing_is_set(monkeypatch):
    monkeypatch.delenv("FFB_PORT", raising=False)
    assert config.engine_port() == 8280


def test_the_data_folder_is_the_configured_one_and_is_created(tmp_path, monkeypatch):
    target = tmp_path / "a" / "b"
    monkeypatch.setenv("FORMFABRIC_DATA_DIR", str(target))
    assert config.data_dir() == target
    assert target.is_dir()


def test_the_data_folder_expands_a_home_shortcut(tmp_path, monkeypatch):
    monkeypatch.setenv("HOME", str(tmp_path))
    monkeypatch.setenv("FORMFABRIC_DATA_DIR", "~/moved")
    assert config.data_dir() == tmp_path / "moved"
    assert (tmp_path / "moved").is_dir()


@pytest.mark.parametrize("raw", ["", "   "])
def test_with_no_data_folder_set_the_app_uses_formfabric_in_the_home_folder(tmp_path, monkeypatch, raw):
    monkeypatch.setenv("HOME", str(tmp_path))
    monkeypatch.setenv("FORMFABRIC_DATA_DIR", raw)
    assert config.data_dir() == tmp_path / "Formfabric"
    assert (tmp_path / "Formfabric").is_dir()


@pytest.mark.parametrize("value,expected", [("1", True), ("0", False), ("true", False), ("", False)])
def test_desktop_mode_is_on_only_for_the_value_1(monkeypatch, value, expected):
    monkeypatch.setenv("FFB_DESKTOP", value)
    assert config.is_desktop() is expected


def test_desktop_mode_is_off_when_unset(monkeypatch):
    monkeypatch.delenv("FFB_DESKTOP", raising=False)
    assert config.is_desktop() is False


def test_the_frontend_build_is_found_only_when_it_has_an_index_page(tmp_path, monkeypatch):
    monkeypatch.setenv("FFB_DIST", str(tmp_path))
    assert config.dist_dir() is None
    (tmp_path / "index.html").write_text("<html></html>")
    assert config.dist_dir() == tmp_path


def test_cors_origins_are_the_configured_list_trimmed_with_blanks_dropped(monkeypatch):
    monkeypatch.setenv("FFB_CORS_ORIGINS", " http://a.test:1 , ,http://b.test ")
    assert config.cors_origins() == ["http://a.test:1", "http://b.test"]


def test_cors_origins_default_to_the_two_local_frontend_addresses(monkeypatch):
    monkeypatch.delenv("FFB_CORS_ORIGINS", raising=False)
    monkeypatch.delattr(sys, "frozen", raising=False)
    port = config.FRONTEND_PORT
    assert config.cors_origins() == [f"http://localhost:{port}", f"http://127.0.0.1:{port}"]


def test_a_packaged_engine_trusts_no_web_origin_unless_told_to(monkeypatch):
    monkeypatch.delenv("FFB_CORS_ORIGINS", raising=False)
    monkeypatch.setattr(sys, "frozen", True, raising=False)
    assert config.is_frozen() is True
    assert config.cors_origins() == []


def test_the_window_origins_hold_the_engine_itself_and_the_desktop_shell(monkeypatch):
    monkeypatch.setenv("FFB_PORT", "9123")
    origins = config.window_origins()
    for wanted in ("http://127.0.0.1:9123", "http://localhost:9123", "tauri://localhost", "http://tauri.localhost"):
        assert wanted in origins


def test_a_packaged_engine_leaves_out_the_preview_server_addresses(monkeypatch):
    monkeypatch.delenv("FFB_CORS_ORIGINS", raising=False)
    monkeypatch.setattr(sys, "frozen", True, raising=False)
    preview = config.FRONTEND_PORT + 2
    assert f"http://localhost:{preview}" not in config.window_origins()
    monkeypatch.delattr(sys, "frozen", raising=False)
    assert f"http://localhost:{preview}" in config.window_origins()


@pytest.mark.parametrize("raw,expected", [("", "5280"), ("9000", "9000"), ("abc", "5280"), ("0", "5280"), ("65533", "65533"), ("65534", "5280"), ("-1", "5280")])
def test_the_frontend_port_is_read_once_at_start_and_bad_values_give_the_documented_5280(raw, expected):
    out = run_python("from app.core import config; print(config.FRONTEND_PORT)", FFB_FRONTEND_PORT=raw)
    assert out == [expected]


def test_the_window_key_is_long_random_and_the_same_for_the_whole_run():
    first = config.window_token()
    assert first == config.window_token()
    assert len(first) >= 43
    other = run_python("from app.core import config; print(config.window_token())")[0]
    assert other != first


def test_the_shell_key_mode_is_on_only_for_the_value_1(monkeypatch):
    monkeypatch.setenv(config.KEY_STDOUT_ENV, "1")
    assert config.key_stdout() is True
    monkeypatch.setenv(config.KEY_STDOUT_ENV, "yes")
    assert config.key_stdout() is False
