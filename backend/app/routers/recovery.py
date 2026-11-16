from fastapi import APIRouter, Depends, HTTPException
from starlette.concurrency import run_in_threadpool

from app.core.window_guard import require_window, require_window_read
from app.services import autosave, engine, recovery

router = APIRouter()


@router.get("/recovery", dependencies=[Depends(require_window_read)])
def state() -> dict:
    pending = None if recovery.foreign() else recovery.pending_info()
    return {**(pending or {"pending": False, "autosave_time": None, "commands": 0}), **recovery.protection()}


def _restore() -> dict:
    if not engine.available():
        raise HTTPException(status_code=503, detail=engine.UNAVAILABLE)
    try:
        result = recovery.restore(lambda name, args: engine.dispatch(name, args))
    except LookupError as exc:
        raise HTTPException(status_code=404, detail=str(exc))
    except RuntimeError as exc:
        raise HTTPException(status_code=422, detail=str(exc))
    autosave.save_now(force=True)
    return result


@router.post("/recovery/restore", dependencies=[Depends(require_window)])
async def restore() -> dict:
    return await run_in_threadpool(_restore)


@router.post("/recovery/discard", dependencies=[Depends(require_window)])
def discard() -> dict:
    return {"discarded": recovery.discard()}
