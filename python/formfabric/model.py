from __future__ import annotations

import time
from typing import Any, Sequence

from .client import Client, default_client

SAMPLES = ("tshirt", "skirt", "dress")
QUALITY = ("fast", "normal", "quality", "final")


class Panel:
    def __init__(self, garment: "Garment", data: dict) -> None:
        self.garment = garment
        self.data = data

    @property
    def id(self) -> int:
        return int(self.data["id"])

    @property
    def name(self) -> str:
        return str(self.data.get("name", ""))

    def __repr__(self) -> str:
        return f"Panel(id={self.id}, name={self.name!r})"

    def rename(self, name: str) -> "Panel":
        self.garment.client.call("pattern.rename", panel=self.id, name=name)
        self.data["name"] = name
        return self

    def measure(self) -> dict:
        return self.garment.client.call("pattern.measure", panel=self.id)

    def positions(self, decimate: int = 1) -> list[list[float]]:
        reply = self.garment.client.call("sim.positions", panel=self.id, decimate=decimate)
        return reply.get("positions", reply) if isinstance(reply, dict) else reply

    def duplicate(self) -> "Panel":
        before = {p.id for p in self.garment.panels}
        self.garment.client.call("pattern.duplicate", panels=[self.id])
        fresh = [p for p in self.garment.refresh() if p.id not in before]
        return fresh[-1] if fresh else self

    def delete(self) -> None:
        self.garment.client.call("pattern.delete", panels=[self.id])
        self.garment.refresh()


class Sim:
    def __init__(self, client: Client) -> None:
        self.client = client

    def reset(self) -> dict:
        return self.client.call("sim.reset")

    def step(self, frames: int = 1) -> dict:
        return self.client.call("sim.step", frames=frames)

    def start(self) -> dict:
        return self.client.call("sim.start")

    def stop(self) -> dict:
        return self.client.call("sim.stop")

    def state(self) -> dict:
        return self.client.call("sim.state")

    def settings(self, **values: Any) -> dict:
        return self.client.call("sim.settings", **values)

    def preset(self, name: str) -> dict:
        if name not in QUALITY:
            raise ValueError(f"preset must be one of {', '.join(QUALITY)}")
        return self.settings(preset=name)

    def drape(self, frames: int = 90, reset: bool = True) -> dict:
        if reset:
            self.reset()
        self.step(frames)
        return self.state()

    def wait(self, frame: int, timeout: float = 60.0) -> dict:
        end = time.monotonic() + timeout
        while True:
            state = self.state()
            if int(state.get("frame", 0)) >= frame or time.monotonic() > end:
                return state
            time.sleep(0.05)


class Garment:
    def __init__(self, client: Client | None = None) -> None:
        self.client = client or default_client()
        self.sim = Sim(self.client)
        self._panels: list[Panel] = []

    def __repr__(self) -> str:
        return f"Garment(panels={len(self._panels)}, backend={self.client.name})"

    @classmethod
    def new(cls, client: Client | None = None) -> "Garment":
        g = cls(client)
        g.client.call("file.new")
        g._panels = []
        return g

    @classmethod
    def sample(cls, name: str = "tshirt", body: str | int | None = None, client: Client | None = None, force: bool = True) -> "Garment":
        if name not in SAMPLES:
            raise ValueError(f"sample must be one of {', '.join(SAMPLES)}")
        g = cls(client)
        args: dict[str, Any] = {"name": name, "force": force}
        if body is not None:
            args["body"] = body
        g.loaded = g.client.call("sample.load", **args)
        g.refresh()
        return g

    @classmethod
    def open(cls, path: str, client: Client | None = None) -> "Garment":
        g = cls(client)
        g.client.call("file.open", path=path)
        g.refresh()
        return g

    def save(self, path: str) -> dict:
        return self.client.call("file.save", path=path)

    def refresh(self) -> list[Panel]:
        doc = self.client.call("pattern.get")
        self._panels = [Panel(self, p) for p in doc.get("panels", [])]
        return self._panels

    @property
    def panels(self) -> list[Panel]:
        return list(self._panels)

    def panel(self, ident: int | str) -> Panel:
        for p in self._panels or self.refresh():
            if p.id == ident or p.name == ident:
                return p
        raise KeyError(f"no panel {ident!r}")

    def rectangle(self, width: float, height: float, name: str | None = None, x: float = 0.0, y: float = 0.0, **more: Any) -> Panel:
        args: dict[str, Any] = {"x": x, "y": y, "width": width, "height": height, **more}
        if name:
            args["name"] = name
        reply = self.client.call("pattern.rectangle", **args)
        self.refresh()
        return self._by_reply(reply, name)

    def draw(self, points: Sequence[Sequence[float]], name: str | None = None, **more: Any) -> Panel:
        args: dict[str, Any] = {"points": [list(p) for p in points], **more}
        if name:
            args["name"] = name
        reply = self.client.call("pattern.draw_panel", **args)
        self.refresh()
        return self._by_reply(reply, name)

    def _by_reply(self, reply: Any, name: str | None) -> Panel:
        ident = None
        if isinstance(reply, dict):
            ident = reply.get("id", reply.get("panel"))
        if ident is not None:
            return self.panel(ident)
        return self._panels[-1]

    def sew(self, a: tuple[Panel | int, int], b: tuple[Panel | int, int], **more: Any) -> Any:
        pa, sa = a
        pb, sb = b
        ids = [p.id if isinstance(p, Panel) else int(p) for p in (pa, pb)]
        return self.client.call("sew.segment", panel_a=ids[0], segment_a=sa, panel_b=ids[1], segment_b=sb, **more)

    def undo(self) -> Any:
        out = self.client.call("history.undo")
        self.refresh()
        return out

    def redo(self) -> Any:
        out = self.client.call("history.redo")
        self.refresh()
        return out

    def export(self, path: str, format: str | None = None, **more: Any) -> dict:
        kind = format or path.rsplit(".", 1)[-1].lower()
        kind = {"usda": "usd"}.get(kind, kind)
        return self.client.call("export.run", format=kind, path=path, **more)

    def export_glb(self, path: str, **more: Any) -> dict:
        return self.export(path, "glb", **more)

    def link(self, folder: str, every_frames: int = 1, **more: Any) -> "Link":
        link = Link(self.client)
        link.start(folder, every_frames, **more)
        return link


class Link:
    def __init__(self, client: Client) -> None:
        self.client = client

    def start(self, folder: str, every_frames: int = 1, **more: Any) -> dict:
        return self.client.call("link.start", folder=folder, every_frames=every_frames, format="usd", **more)

    def stop(self) -> dict:
        return self.client.call("link.stop")

    def state(self) -> dict:
        return self.client.call("link.state")

    def __enter__(self) -> "Link":
        return self

    def __exit__(self, *_exc: object) -> None:
        self.stop()


def sample_drape(name: str = "tshirt", frames: int = 60, client: Client | None = None, body: str | int | None = None) -> Garment:
    g = Garment.sample(name, body=body, client=client)
    g.sim.drape(frames)
    return g
