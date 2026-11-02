from __future__ import annotations

import inspect
import json
import keyword
import re
from pathlib import Path
from types import SimpleNamespace
from typing import Any, Callable

from . import schema as _schema
from .client import Client, default_client
from .errors import ArgumentError

SNAPSHOT = Path(__file__).with_name("catalogue.json")


class _Unset:
    def __repr__(self) -> str:
        return "UNSET"

    def __bool__(self) -> bool:
        return False


UNSET = _Unset()


def function_name(command_id: str) -> str:
    name = re.sub(r"[^0-9a-zA-Z_]", "_", command_id)
    return name + "_" if keyword.iskeyword(name) or name[:1].isdigit() else name


def _python_name(arg: str) -> str:
    name = re.sub(r"[^0-9a-zA-Z_]", "_", arg)
    return name + "_" if keyword.iskeyword(name) or name[:1].isdigit() else name


def _annotation(spec: dict) -> Any:
    kinds = [_schema.PYTHON_TYPES[t] for t in _schema.types_of(spec) if t in _schema.PYTHON_TYPES]
    if len(kinds) == 1:
        return kinds[0]
    return Any


def _describe_arg(name: str, spec: dict, required: bool) -> str:
    kinds = "/".join(_schema.types_of(spec)) or "any"
    bits = [kinds, "required" if required else "optional"]
    if "enum" in spec:
        bits.append("one of " + ", ".join(repr(o) for o in spec["enum"]))
    for key in ("minimum", "maximum"):
        if key in spec:
            bits.append(f"{key} {spec[key]}")
    if "default" in spec:
        bits.append(f"default {spec['default']!r}")
    text = str(spec.get("description", "")).strip()
    return f"    {name} ({', '.join(bits)}): {text}".rstrip()


def _docstring(spec: dict) -> str:
    props = (spec.get("inputSchema") or {}).get("properties") or {}
    required = set((spec.get("inputSchema") or {}).get("required") or [])
    lines = [f"{spec['label']} ({spec['id']}, MCP tool {spec['name']}, group {spec['group']}).", "", str(spec["description"]).strip()]
    hints = [k for k, v in (spec.get("annotations") or {}).items() if v]
    if hints:
        lines += ["", "Hints: " + ", ".join(hints) + "."]
    if props:
        lines += ["", "Arguments:"]
        lines += [_describe_arg(_python_name(k), v, k in required) for k, v in props.items()]
    lines += ["", "Pass client=Client to use a client other than the default."]
    return "\n".join(lines)


def make_function(spec: dict, getter: Callable[[], Client]) -> Callable[..., Any]:
    command = spec["id"]
    input_schema = spec.get("inputSchema") or {}
    props = input_schema.get("properties") or {}
    required = set(input_schema.get("required") or [])
    names = {key: _python_name(key) for key in props}
    reverse = {v: k for k, v in names.items()}
    params = []
    for key, prop in props.items():
        default = inspect.Parameter.empty if key in required else UNSET
        params.append(inspect.Parameter(names[key], inspect.Parameter.KEYWORD_ONLY, default=default, annotation=_annotation(prop)))
    if "client" not in reverse:
        params.append(inspect.Parameter("client", inspect.Parameter.KEYWORD_ONLY, default=None, annotation=Client | None))
    accepts_client = "client" not in reverse

    def call(**kwargs: Any) -> Any:
        client = kwargs.pop("client", None) if accepts_client else None
        args = {}
        for given, value in kwargs.items():
            if value is UNSET:
                continue
            key = reverse.get(given)
            if key is None:
                raise ArgumentError(command, f"unknown argument {given}")
            args[key] = value
        return (client or getter()).run(command, args)

    call.__name__ = function_name(command)
    call.__qualname__ = call.__name__
    call.__doc__ = _docstring(spec)
    call.__signature__ = inspect.Signature(params, return_annotation=Any)
    call.command = command
    call.spec = spec
    return call


class Commands:
    def __init__(self, getter: Callable[[], Client], catalogue: dict) -> None:
        self._getter = getter
        self.functions: dict[str, Callable[..., Any]] = {}
        groups: dict[str, SimpleNamespace] = {}
        for spec in catalogue["commands"]:
            fn = make_function(spec, getter)
            self.functions[spec["id"]] = fn
            setattr(self, fn.__name__, fn)
            head, _, tail = spec["id"].partition(".")
            if tail:
                setattr(groups.setdefault(head, SimpleNamespace()), _python_name(tail), fn)
        for head, space in groups.items():
            if not hasattr(self, head):
                setattr(self, head, space)

    def __iter__(self):
        return iter(self.functions.values())

    def __len__(self) -> int:
        return len(self.functions)

    def __getitem__(self, command: str) -> Callable[..., Any]:
        return self.functions[command]


def load_snapshot() -> dict:
    if not SNAPSHOT.is_file():
        return {"commands": []}
    return json.loads(SNAPSHOT.read_text(encoding="utf-8"))


def write_snapshot(catalogue: dict) -> Path:
    SNAPSHOT.write_text(json.dumps(catalogue, indent=1, sort_keys=True) + "\n", encoding="utf-8")
    return SNAPSHOT


_table = Commands(default_client, load_snapshot())
COMMANDS: dict[str, Callable[..., Any]] = _table.functions
__all__ = ["COMMANDS", "Commands", "UNSET", "function_name", "load_snapshot", "make_function", "write_snapshot"] + [fn.__name__ for fn in _table]
for _fn in _table:
    if _fn.__name__ not in globals():
        globals()[_fn.__name__] = _fn
for _head in [k for k, v in vars(_table).items() if isinstance(v, SimpleNamespace)]:
    if _head not in globals():
        globals()[_head] = getattr(_table, _head)
        __all__.append(_head)
