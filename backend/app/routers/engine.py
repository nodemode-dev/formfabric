from typing import Any

from fastapi import APIRouter, Depends, HTTPException
from pydantic import BaseModel, Field

from app.core.window_guard import require_window, require_window_read
from app.services import engine

router = APIRouter()


class Call(BaseModel):
    name: str = Field(max_length=120)
    args: dict[str, Any] = Field(default_factory=dict)


def _guard() -> None:
    if not engine.available():
        raise HTTPException(status_code=503, detail=engine.UNAVAILABLE)


@router.get("/catalogue")
def catalogue() -> dict:
    _guard()
    return engine.catalogue()


@router.post("/dispatch", dependencies=[Depends(require_window)])
def dispatch(call: Call) -> dict:
    _guard()
    return engine.dispatch(call.name, call.args)


@router.get("/stream", dependencies=[Depends(require_window_read)])
def stream() -> dict:
    _guard()
    return engine.stream_info()
