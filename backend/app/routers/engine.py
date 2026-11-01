from typing import Any

from fastapi import APIRouter, HTTPException
from pydantic import BaseModel, Field

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


@router.post("/dispatch")
def dispatch(call: Call) -> dict:
    _guard()
    return engine.dispatch(call.name, call.args)


@router.get("/stream")
def stream() -> dict:
    _guard()
    return engine.stream_info()
