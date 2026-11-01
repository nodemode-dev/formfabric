import json
import os
import sys


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


def parent_alive(pid: int) -> bool:
    if os.name == "nt":
        import ctypes

        kernel32 = ctypes.windll.kernel32
        handle = kernel32.OpenProcess(0x00100000, False, pid)
        if not handle:
            return False
        try:
            return kernel32.WaitForSingleObject(handle, 0) == 0x102
        finally:
            kernel32.CloseHandle(handle)
    try:
        os.kill(pid, 0)
    except ProcessLookupError:
        return False
    except PermissionError:
        return True
    return True


def watch_parent() -> None:
    raw = os.environ.get("FFB_PARENT_PID", "")
    if not raw.isdigit():
        return
    pid = int(raw)
    import threading
    import time

    def loop() -> None:
        while parent_alive(pid):
            time.sleep(1.5)
        os._exit(0)

    threading.Thread(target=loop, name="parent-watch", daemon=True).start()


def free_port() -> int:
    import socket

    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.bind(("127.0.0.1", 0))
        return s.getsockname()[1]


def main() -> int:
    if "--selfcheck" in sys.argv:
        return selfcheck()
    watch_parent()
    bundled = os.path.join(resource_dir(), "frontend_dist")
    if os.path.isdir(bundled):
        os.environ.setdefault("FFB_DIST", bundled)
    os.environ.setdefault("FFB_DESKTOP", "1")
    import uvicorn

    from app.main import app as fastapi_app

    port = int(os.environ.get("FFB_PORT", "0") or 0) or free_port()
    os.environ["FFB_PORT"] = str(port)
    uvicorn.run(fastapi_app, host="127.0.0.1", port=port, log_level="info")
    return 0


if __name__ == "__main__":
    import multiprocessing

    multiprocessing.freeze_support()
    sys.exit(main())
