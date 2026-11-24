import io
import json
import sys
import types
import urllib.error
from pathlib import Path

import pytest

REPO = Path(__file__).resolve().parents[3]
PYTHON_DIR = REPO / "python"
if str(PYTHON_DIR) not in sys.path:
    sys.path.insert(0, str(PYTHON_DIR))

import formfabric  # noqa: E402
from formfabric import backend as ffb_backend  # noqa: E402

CATALOGUE_FILE = PYTHON_DIR / "formfabric" / "catalogue.json"
CATALOGUE = json.loads(CATALOGUE_FILE.read_text(encoding="utf-8"))
SPECS = {s["id"]: s for s in CATALOGUE["commands"]}
TOKEN = "t" * 32


class Fail:
    def __init__(self, code="boom", message="it broke"):
        self.code = code
        self.message = message


class FakeEngine:
    def __init__(self, replies=None):
        self.calls = []
        self.replies = dict(replies or {})
        self.panels = []
        self.next_id = 1
        self.frame = 0

    def names(self):
        return [name for name, _ in self.calls]

    def args_of(self, name):
        return [a for n, a in self.calls if n == name]

    def add_panel(self, name):
        pid = self.next_id
        self.next_id += 1
        self.panels.append({"id": pid, "name": name or f"Panel {pid}"})
        return pid

    def handle(self, name, args):
        if name in self.replies:
            reply = self.replies[name]
            return reply(args) if callable(reply) else reply
        if name == "file.new":
            self.panels = []
            return {}
        if name == "sample.load":
            self.panels = []
            self.add_panel("Front")
            self.add_panel("Back")
            return {"panels": 2}
        if name == "pattern.get":
            return {"panels": [dict(p) for p in self.panels]}
        if name in ("pattern.rectangle", "pattern.draw_panel"):
            return {"id": self.add_panel(args.get("name"))}
        if name == "pattern.rename":
            for p in self.panels:
                if p["id"] == args["panel"]:
                    p["name"] = args["name"]
            return {}
        if name == "pattern.delete":
            self.panels = [p for p in self.panels if p["id"] not in args["panels"]]
            return {}
        if name == "pattern.duplicate":
            for p in list(self.panels):
                if p["id"] in args["panels"]:
                    self.add_panel(p["name"] + " copy")
            return {}
        if name == "sim.reset":
            self.frame = 0
            return {"frame": 0}
        if name == "sim.step":
            self.frame += args.get("frames", 1)
            return {"frame": self.frame}
        if name == "sim.state":
            return {"frame": self.frame}
        return {}

    def dispatch(self, name, raw):
        args = json.loads(raw)
        self.calls.append((name, args))
        out = self.handle(name, args)
        if isinstance(out, Fail):
            return json.dumps({"ok": False, "error": {"code": out.code, "message": out.message}})
        return json.dumps({"ok": True, "result": out})


def engine_module(engine, catalogue=None):
    module = types.ModuleType("ffb_engine")
    module.Engine = lambda: engine
    module.catalogue = lambda: json.dumps(catalogue if catalogue is not None else CATALOGUE)
    module.version = lambda: "0.0.0"
    return module


@pytest.fixture
def engine(monkeypatch):
    fake = FakeEngine()
    monkeypatch.setitem(sys.modules, "ffb_engine", engine_module(fake))
    return fake


@pytest.fixture
def client(engine):
    c = formfabric.connect("engine")
    yield c
    c.close()


@pytest.fixture
def no_default_client():
    formfabric.use(None)
    yield
    formfabric.use(None)


@pytest.fixture
def clean_env(monkeypatch):
    import os

    for key in [k for k in os.environ if k.startswith("FFB_") or k == "FORMSCULPT_GOF"]:
        monkeypatch.delenv(key)


class FakeResponse:
    def __init__(self, payload):
        self.raw = payload if isinstance(payload, bytes) else json.dumps(payload).encode("utf-8")

    def read(self):
        return self.raw

    def __enter__(self):
        return self

    def __exit__(self, *_):
        return False


class FakeOpener:
    def __init__(self, responder=None):
        self.requests = []
        self.responder = responder or (lambda path, body: {"ok": True, "result": {}})

    def open(self, request, timeout=None):
        body = json.loads(request.data.decode("utf-8")) if request.data else None
        path = request.full_url.split("/", 3)[3] if request.full_url.count("/") >= 3 else ""
        self.requests.append(
            {
                "url": request.full_url,
                "method": request.get_method(),
                "headers": {k.lower(): v for k, v in request.header_items()},
                "body": body,
                "timeout": timeout,
                "path": "/" + path,
            }
        )
        out = self.responder("/" + path, body)
        if isinstance(out, Exception):
            raise out
        return FakeResponse(out)


def http_error(code, detail=None, url="http://127.0.0.1:8280/x"):
    body = json.dumps({"detail": detail}).encode("utf-8") if detail is not None else b"not json"
    return urllib.error.HTTPError(url, code, "status", {}, io.BytesIO(body))


@pytest.fixture
def opener(monkeypatch, clean_env):
    fake = FakeOpener()
    monkeypatch.setattr(ffb_backend, "_OPENER", fake)
    return fake
