from __future__ import annotations

import difflib
from typing import Any

from . import schema
from .backend import Backend, make, unwrap
from .errors import ArgumentError


class Client:
    def __init__(self, backend: Backend, check: bool = True) -> None:
        self.backend = backend
        self.check = check
        self._specs: dict[str, dict] | None = None

    @property
    def name(self) -> str:
        return self.backend.name

    def catalogue(self) -> dict:
        return self.backend.catalogue()

    def specs(self) -> dict[str, dict]:
        if self._specs is None:
            table: dict[str, dict] = {}
            for spec in self.catalogue()["commands"]:
                table[spec["id"]] = spec
                table[spec["name"]] = spec
            self._specs = table
        return self._specs

    def spec(self, name: str) -> dict:
        found = self.specs().get(name)
        if found is None:
            near = difflib.get_close_matches(name, [s for s in self.specs() if "." in s or "_" in s], n=3)
            hint = f"; did you mean {', '.join(near)}?" if near else ""
            raise ArgumentError(name, f"no such command{hint}")
        return found

    def raw(self, name: str, args: dict | None = None) -> dict:
        return self.backend.dispatch(name, args or {})

    def call(self, name: str, /, **args: Any) -> Any:
        return self.run(name, args)

    def run(self, name: str, args: dict | None = None) -> Any:
        args = {k: v for k, v in (args or {}).items()}
        if self.check:
            schema.check(name, self.spec(name)["inputSchema"], args)
        return unwrap(name, self.backend.dispatch(name, args))

    def bind(self):
        from .commands import Commands

        return Commands(lambda: self, self.catalogue())

    def close(self) -> None:
        self.backend.close()

    def __enter__(self) -> "Client":
        return self

    def __exit__(self, *_exc: object) -> None:
        self.close()


_default: Client | None = None


def connect(backend: str = "auto", url: str | None = None, token: str | None = None, check: bool = True) -> Client:
    return Client(make(backend, url, token), check)


def default_client() -> Client:
    global _default
    if _default is None:
        _default = connect()
    return _default


def use(client: Client | None) -> None:
    global _default
    _default = client
