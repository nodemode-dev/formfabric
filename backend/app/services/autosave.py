import asyncio
import os

from starlette.concurrency import run_in_threadpool

from app.core import config
from app.services import engine

INTERVAL = float(os.environ.get("FFB_AUTOSAVE_SECONDS", "180"))
_saved_epoch = 0


def path() -> str:
    folder = config.data_dir() / "autosave"
    folder.mkdir(parents=True, exist_ok=True)
    return str(folder / "autosave.ffbrc")


def save_now() -> bool:
    global _saved_epoch
    current = engine.epoch()
    if current == _saved_epoch:
        return False
    reply = engine.dispatch("file.save", {"path": path()}, notify=False)
    if not reply.get("ok"):
        return False
    _saved_epoch = engine.epoch()
    return True


async def loop() -> None:
    while True:
        await asyncio.sleep(INTERVAL)
        if engine.available():
            try:
                await run_in_threadpool(save_now)
            except OSError:
                pass
