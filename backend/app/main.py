import asyncio
import mimetypes
from contextlib import asynccontextmanager

from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
from fastapi.staticfiles import StaticFiles

from app.core import config, quiet_log
from app.core.isolation import CrossOriginIsolation
from app.core.local_guard import LocalGuard
from app.routers import commands, engine, events, mcp, plugins, recovery as recovery_router, system, terminal_socket
from app.services import autosave, claude_code, cmd_chat, cmd_terminal, recovery, terminal
from app.services import events as bus

mimetypes.add_type("text/javascript", ".js")
mimetypes.add_type("text/javascript", ".mjs")
mimetypes.add_type("text/css", ".css")
mimetypes.add_type("application/wasm", ".wasm")

WINDOW_COMMANDS = (cmd_terminal, claude_code, cmd_chat)


@asynccontextmanager
async def lifespan(_: FastAPI):
    quiet_log.install()
    loop = asyncio.get_running_loop()
    bus.bind(loop)
    terminal.start(loop)
    recovery.start()
    task = asyncio.create_task(autosave.loop())
    yield
    task.cancel()
    recovery.close()
    terminal.close_all()


app = FastAPI(title=config.APP_NAME, version=config.VERSION, lifespan=lifespan)
app.add_middleware(CORSMiddleware, allow_origins=config.cors_origins(), allow_methods=["*"], allow_headers=["*"])
app.add_middleware(CrossOriginIsolation)
app.add_middleware(LocalGuard, origins=config.window_origins())
app.include_router(system.router, prefix="/api")
app.include_router(engine.router, prefix="/api/engine")
app.include_router(mcp.router, prefix="/api")
app.include_router(plugins.router, prefix="/api")
app.include_router(commands.router, prefix="/api")
app.include_router(events.router, prefix="/api")
app.include_router(recovery_router.router, prefix="/api")
app.include_router(terminal_socket.router)

dist = config.dist_dir()
if dist is not None:
    app.mount("/", StaticFiles(directory=dist, html=True), name="frontend")
