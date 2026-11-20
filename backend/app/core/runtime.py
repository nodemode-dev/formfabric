import copy
import json
import logging
import os
from logging.handlers import RotatingFileHandler
from pathlib import Path

from app.core import startup

LOG_NAME = "formfabric-engine.log"
LOG_BYTES = 1_048_576
LOG_KEEP = 3
FORMAT = "%(asctime)s %(levelname)s %(name)s: %(message)s"
AlreadyRunning = startup.Busy


def log_path(data: Path) -> Path:
    folder = data / "Logs"
    folder.mkdir(parents=True, exist_ok=True)
    return folder / LOG_NAME


def start_logging(data: Path) -> Path:
    path = log_path(data)
    handler = RotatingFileHandler(path, maxBytes=LOG_BYTES, backupCount=LOG_KEEP, encoding="utf-8")
    handler.setFormatter(logging.Formatter(FORMAT))
    log = logging.getLogger("formfabric")
    log.setLevel(logging.INFO)
    log.addHandler(handler)
    log.info("engine started, pid %s, data folder %s", os.getpid(), data)
    return path


def server_log_config(data: Path) -> dict:
    from uvicorn.config import LOGGING_CONFIG

    config = copy.deepcopy(LOGGING_CONFIG)
    for handler in config["handlers"].values():
        handler["stream"] = "ext://sys.stdout"
    for formatter in config["formatters"].values():
        formatter["use_colors"] = False
    config["formatters"]["file"] = {"format": FORMAT}
    config["handlers"]["file"] = {"class": "logging.handlers.RotatingFileHandler", "filename": str(log_path(data)), "maxBytes": LOG_BYTES, "backupCount": LOG_KEEP, "encoding": "utf-8", "formatter": "file"}
    for name in ("uvicorn", "uvicorn.access"):
        config["loggers"][name]["handlers"] = [*config["loggers"][name]["handlers"], "file"]
    return config


def claim(data: Path) -> Path:
    lock, replaced = startup.claim(data, startup.shell_pid())
    if replaced:
        logging.getLogger("formfabric").warning("an engine left running without its app was stopped (process %s)", replaced)
    return lock


def release(data: Path) -> None:
    lock = data / startup.LOCK_NAME
    try:
        if int(json.loads(lock.read_text(encoding="utf-8")).get("pid", 0)) == os.getpid():
            lock.unlink()
    except (OSError, ValueError, AttributeError):
        return
