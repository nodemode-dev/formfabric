import asyncio
import json

from fastapi import APIRouter, WebSocket, WebSocketDisconnect

from app.core import window_guard
from app.services import terminal

router = APIRouter(tags=["terminal"])
BATCH = 128 * 1024


@router.websocket("/api/terminal/{sid}")
async def attach(ws: WebSocket, sid: str) -> None:
    await ws.accept()
    if not await window_guard.socket_from_window(ws):
        await ws.close(code=4403)
        return
    try:
        session = terminal.get(sid)
    except KeyError:
        await ws.close(code=4404)
        return
    q = terminal.Subscriber(session.pos)
    session.subscribers.add(q)

    async def outbound() -> None:
        backlog, seen = session.snapshot()
        if backlog:
            await ws.send_text(backlog)
        if not session.alive:
            await ws.send_text(json.dumps({"type": "exit"}))
        while True:
            item = await q.get()
            parts: list[str] = []
            size = 0
            top = 0
            ended = False
            while True:
                if item is None:
                    ended = True
                    break
                text, pos = item
                if pos == -2:
                    await ws.close(code=1013)
                    return
                if pos < 0 or pos > seen:
                    parts.append(text)
                    size += len(text)
                top = max(top, pos)
                if size >= BATCH or q.empty():
                    break
                item = q.get_nowait()
            if parts:
                await ws.send_text("".join(parts))
            q.consumed(top)
            if ended:
                await ws.send_text(json.dumps({"type": "exit"}))

    sender = asyncio.create_task(outbound())
    try:
        while True:
            raw = await ws.receive_text()
            try:
                msg = json.loads(raw)
            except ValueError:
                continue
            if msg.get("type") == "input" and session.alive:
                terminal.write(sid, str(msg.get("data", "")))
            elif msg.get("type") == "resize":
                try:
                    terminal.resize(sid, int(msg.get("cols", 80)), int(msg.get("rows", 24)))
                except (TypeError, ValueError):
                    continue
    except WebSocketDisconnect:
        pass
    finally:
        sender.cancel()
        session.subscribers.discard(q)
