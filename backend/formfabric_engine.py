import json
import logging
import os
import sys
from pathlib import Path

if sys.stdin is None:
    sys.stdin = open(os.devnull, encoding="utf-8")
if sys.stdout is None:
    sys.stdout = open(os.devnull, "w", encoding="utf-8")
if sys.stderr is None:
    sys.stderr = open(os.devnull, "w", encoding="utf-8")


def resource_dir() -> str:
    return getattr(sys, "_MEIPASS", os.path.dirname(os.path.abspath(__file__)))


def selfcheck() -> int:
    info = {"python": sys.version.split()[0]}
    failed = False
    for name in ("fastapi", "uvicorn", "websockets", "pydantic"):
        try:
            module = __import__(name)
            info[name] = getattr(module, "__version__", "ok")
        except Exception as exc:
            info[name] = f"missing: {exc}"
            failed = True
    try:
        from app import __version__
        from app import main, mcp_stdio
        from app.services import engine, terminal

        info["formfabric"] = __version__
        info["app"] = bool(main.app and mcp_stdio.handle)
        info["terminal"] = bool(terminal.login_shell())
        info["engine_built"] = engine.available()
        if engine.available():
            info["engine"] = engine.status()["version"]
            info["engine_commands"] = len(engine.catalogue()["commands"])
    except Exception as exc:
        info["app"] = f"failed: {type(exc).__name__}: {exc}"
        failed = True
    info["dist_bundled"] = os.path.isdir(os.path.join(resource_dir(), "frontend_dist"))
    print(json.dumps(info))
    return 1 if failed else 0


def watch_shell(data) -> None:
    from app.core import runtime, startup

    shell = startup.shell_pid()
    if shell <= 1:
        return

    def leave() -> None:
        logging.getLogger("formfabric").info("the app that started this engine is gone (process %s), stopping", shell)
        runtime.release(data)
        os._exit(0)

    startup.watch(shell, leave)


def free_port() -> int:
    import socket

    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.bind(("127.0.0.1", 0))
        return s.getsockname()[1]


def main() -> int:
    if "--selfcheck" in sys.argv:
        return selfcheck()
    if len(sys.argv) > 1 and sys.argv[1] == "mcp":
        from app import mcp_stdio

        return mcp_stdio.main()
    if len(sys.argv) > 1 and sys.argv[1] == "agents":
        import runpy

        tool = os.path.join(resource_dir(), "setup", "agents.py")
        if not os.path.isfile(tool):
            print("formfabric: the agent setup tool is not in this build", file=sys.stderr)
            return 1
        return runpy.run_path(tool, run_name="__formfabric_agents__")["main"](sys.argv[2:])
    return serve()


def serve() -> int:
    from app.core import config, runtime, startup

    try:
        data = config.data_dir()
        logs = runtime.start_logging(data).parent
    except OSError as err:
        print(f"formfabric cannot use its data folder: {err}", file=sys.stderr)
        return 5
    startup.keep_errors(logs)
    log = logging.getLogger("formfabric")
    if startup.alone():
        beside = startup.app_beside(Path(sys.executable))
        if beside is not None and startup.open_app(beside):
            log.info("started without its app, so it opened %s and stopped", beside)
            return 0
        log.error("started without its app, and no app program was found near %s", sys.executable)
        print("formfabric-engine is a part of FormFabric and serves nothing on its own. Open FormFabric instead.", file=sys.stderr)
        return 4
    try:
        runtime.claim(data)
    except runtime.AlreadyRunning as err:
        log.error("%s", err)
        print(err, file=sys.stderr)
        return 2
    watch_shell(data)
    bundled = os.path.join(resource_dir(), "frontend_dist")
    if os.path.isdir(bundled):
        os.environ.setdefault("FFB_DIST", bundled)
    os.environ.setdefault("FFB_DESKTOP", "1")
    import uvicorn

    from app.main import app as fastapi_app

    port = int(os.environ.get("FFB_PORT", "0") or 0) or free_port()
    os.environ["FFB_PORT"] = str(port)
    server = uvicorn.Server(uvicorn.Config(fastapi_app, host="127.0.0.1", port=port, log_level="info", log_config=runtime.server_log_config(data), timeout_graceful_shutdown=1))
    try:
        server.run()
    finally:
        runtime.release(data)
    return 0 if server.started else 3


if __name__ == "__main__":
    import multiprocessing

    multiprocessing.freeze_support()
    sys.exit(main())
