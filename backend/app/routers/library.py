import os

from fastapi import APIRouter, Depends, HTTPException, Request
from fastapi.responses import FileResponse
from starlette.concurrency import run_in_threadpool

from app.core.window_guard import require_window, require_window_read
from app.services import library

router = APIRouter()
NO_CACHE = {"Cache-Control": "no-cache"}


def _fail(exc: Exception) -> HTTPException:
    if isinstance(exc, LookupError):
        return HTTPException(status_code=404, detail=str(exc.args[0]) if exc.args else "Not found.")
    if isinstance(exc, (library.NotInLibrary, ValueError)):
        return HTTPException(status_code=400, detail=str(exc))
    return HTTPException(status_code=500, detail=f"The library folder could not be changed: {exc}")


def _run(work):
    try:
        return work()
    except (LookupError, ValueError, library.NotInLibrary, OSError) as exc:
        raise _fail(exc)


@router.get("/garments", dependencies=[Depends(require_window_read)])
def list_garments() -> dict:
    return _run(lambda: {"garments": library.garments(), "library": str(library.library_dir()), "untitled": library.untitled_path(), **library.settings()})


@router.post("/garments", dependencies=[Depends(require_window)])
def create_garment(body: dict) -> dict:
    template = body.get("template")
    title = body.get("title")
    if not isinstance(template, str) or not template:
        raise HTTPException(status_code=400, detail="A template is needed; a blank garment is File, New Project.")
    if title is not None and not isinstance(title, str):
        raise HTTPException(status_code=400, detail="title must be text")
    return _run(lambda: library.create(template, title))


@router.post("/garments/settings", dependencies=[Depends(require_window)])
def set_settings(body: dict) -> dict:
    return _run(lambda: library.update_settings(body))


@router.post("/garments/import", dependencies=[Depends(require_window)])
async def import_file(request: Request, name: str) -> dict:
    try:
        kind = library.import_kind(name)
    except ValueError as exc:
        raise _fail(exc)
    limit = library.import_limit(kind)
    declared = request.headers.get("content-length", "")
    if declared.isdigit() and int(declared) > limit:
        raise HTTPException(status_code=413, detail="That file is too large to bring in.")
    try:
        fd, tmp = library.import_temp(kind)
    except OSError as exc:
        raise _fail(exc)
    size = 0
    try:
        with os.fdopen(fd, "wb") as handle:
            async for chunk in request.stream():
                size += len(chunk)
                if size > limit:
                    raise HTTPException(status_code=413, detail="That file is too large to bring in.")
                handle.write(chunk)
            handle.flush()
            os.fsync(handle.fileno())
        if size == 0:
            raise HTTPException(status_code=400, detail="That file is empty.")
    except BaseException:
        try:
            os.unlink(tmp)
        except OSError:
            pass
        raise
    return await run_in_threadpool(_run, lambda: library.finish_import(kind, name, tmp))


@router.put("/garments/thumb", dependencies=[Depends(require_window)])
async def put_thumb(request: Request, path: str) -> dict:
    declared = request.headers.get("content-length", "")
    if declared.isdigit() and int(declared) > library.THUMB_LIMIT:
        raise HTTPException(status_code=413, detail="A preview is at most 2 MB.")
    data = bytearray()
    async for chunk in request.stream():
        data.extend(chunk)
        if len(data) > library.THUMB_LIMIT:
            raise HTTPException(status_code=413, detail="A preview is at most 2 MB.")
    gid = await run_in_threadpool(_run, lambda: library.put_thumb(path, bytes(data)))
    return {"ok": True, "id": gid}


@router.get("/garments/trash", dependencies=[Depends(require_window_read)])
def list_trash() -> dict:
    return _run(lambda: {"garments": library.trash()})


@router.delete("/garments/trash", dependencies=[Depends(require_window)])
def empty_trash() -> dict:
    return _run(lambda: {"ok": True, "removed": library.empty_trash()})


@router.post("/garments/trash/{entry}/restore", dependencies=[Depends(require_window)])
def restore(entry: str) -> dict:
    return _run(lambda: library.restore(entry))


@router.delete("/garments/trash/{entry}", dependencies=[Depends(require_window)])
def purge(entry: str) -> dict:
    _run(lambda: library.purge(entry))
    return {"ok": True}


@router.post("/garments/{gid}/duplicate", dependencies=[Depends(require_window)])
def duplicate(gid: str) -> dict:
    return _run(lambda: library.duplicate(gid))


@router.post("/garments/{gid}/forget", dependencies=[Depends(require_window)])
def forget(gid: str) -> dict:
    _run(lambda: library.forget(gid))
    return {"ok": True}


@router.get("/garments/{gid}/thumb", dependencies=[Depends(require_window_read)])
def get_thumb(gid: str):
    picture, kind = _run(lambda: library.thumb(gid))
    return FileResponse(picture, media_type=kind, headers=NO_CACHE)


@router.delete("/garments/{gid}", dependencies=[Depends(require_window)])
def delete(gid: str) -> dict:
    entry = _run(lambda: library.move_to_trash(gid))
    return {"ok": True, "entry": entry}


@router.get("/garment-templates", dependencies=[Depends(require_window_read)])
def list_templates() -> dict:
    return _run(library.templates)


@router.get("/garment-templates/{tid}/thumb", dependencies=[Depends(require_window_read)])
def template_thumb(tid: str):
    picture = _run(lambda: library.template_thumb(tid))
    return FileResponse(picture, media_type="image/webp", headers=NO_CACHE)
