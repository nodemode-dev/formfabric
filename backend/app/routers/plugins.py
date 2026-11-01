from fastapi import APIRouter, HTTPException
from fastapi.concurrency import run_in_threadpool
from pydantic import BaseModel

from app.services import engine, scripting

router = APIRouter()


class RunRequest(BaseModel):
    name: str
    args: dict = {}


@router.get("/plugins")
def plugins() -> dict:
    return {"plugins": [{k: p[k] for k in ("name", "title", "description", "params", "click")} for p in scripting.list_plugins()]}


@router.post("/plugins/run")
async def run(body: RunRequest) -> dict:
    plugin = next((p for p in scripting.list_plugins() if p["name"] == body.name), None)
    if plugin is None:
        raise HTTPException(status_code=404, detail=f"no plugin {body.name}")
    ffb = scripting.Ffb(engine.dispatch)
    try:
        ran = await run_in_threadpool(scripting.run_file, plugin["path"], ffb, body.args)
    except scripting.ScriptError as err:
        return {"ok": False, "error": {"code": "plugin_failed", "message": str(err)}}
    except Exception as err:
        return {"ok": False, "error": {"code": "plugin_error", "message": f"{type(err).__name__}: {err}"}}
    result = ran["result"]
    return {"ok": True, "result": result if isinstance(result, (dict, list, str, int, float, bool, type(None))) else str(result), "output": ran["output"]}
