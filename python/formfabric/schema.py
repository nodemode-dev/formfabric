from __future__ import annotations

import re
from typing import Any

from .errors import ArgumentError

_TYPES = {
    "string": lambda v: isinstance(v, str),
    "boolean": lambda v: isinstance(v, bool),
    "integer": lambda v: (isinstance(v, int) and not isinstance(v, bool)) or (isinstance(v, float) and v.is_integer()),
    "number": lambda v: isinstance(v, (int, float)) and not isinstance(v, bool),
    "array": lambda v: isinstance(v, (list, tuple)),
    "object": lambda v: isinstance(v, dict),
    "null": lambda v: v is None,
}

PYTHON_TYPES = {"string": str, "boolean": bool, "integer": int, "number": float, "array": list, "object": dict}


def types_of(spec: dict) -> list[str]:
    kind = spec.get("type")
    if isinstance(kind, str):
        return [kind]
    if isinstance(kind, list):
        return [k for k in kind if isinstance(k, str)]
    return []


def _fits(types: list[str], value: Any) -> bool:
    return not types or any(_TYPES.get(t, lambda _v: True)(value) for t in types)


def _check_value(command: str, path: str, spec: dict, value: Any) -> None:
    types = types_of(spec)
    if not _fits(types, value):
        raise ArgumentError(command, f"{path} must be {' or '.join(types)}, got {type(value).__name__}")
    options = spec.get("enum")
    if isinstance(options, list) and value not in options:
        raise ArgumentError(command, f"{path} must be one of {', '.join(repr(o) for o in options)}")
    if isinstance(value, (int, float)) and not isinstance(value, bool):
        low, high = spec.get("minimum"), spec.get("maximum")
        if isinstance(low, (int, float)) and value < low:
            raise ArgumentError(command, f"{path} must be at least {low}")
        if isinstance(high, (int, float)) and value > high:
            raise ArgumentError(command, f"{path} must be at most {high}")
    if isinstance(value, str):
        low, high = spec.get("minLength"), spec.get("maxLength")
        if isinstance(low, int) and len(value) < low:
            raise ArgumentError(command, f"{path} must have at least {low} characters")
        if isinstance(high, int) and len(value) > high:
            raise ArgumentError(command, f"{path} must have at most {high} characters")
        pattern = spec.get("pattern")
        if isinstance(pattern, str) and re.search(pattern, value) is None:
            raise ArgumentError(command, f"{path} must match {pattern}")
    if isinstance(value, (list, tuple)):
        low, high = spec.get("minItems"), spec.get("maxItems")
        if isinstance(low, int) and len(value) < low:
            raise ArgumentError(command, f"{path} needs at least {low} items")
        if isinstance(high, int) and len(value) > high:
            raise ArgumentError(command, f"{path} takes at most {high} items")
        item = spec.get("items")
        if isinstance(item, dict):
            for i, element in enumerate(value):
                _check_value(command, f"{path}[{i}]", item, element)
    if isinstance(value, dict) and isinstance(spec.get("properties"), dict):
        check_object(command, spec, value, path)


def check_object(command: str, schema: dict, args: dict, path: str = "") -> None:
    props = schema.get("properties") or {}
    prefix = f"{path}." if path else ""
    for key in schema.get("required") or []:
        if key not in args:
            raise ArgumentError(command, f"missing required argument {prefix}{key}")
    closed = schema.get("additionalProperties") is False
    for key, value in args.items():
        spec = props.get(key)
        if spec is None:
            if closed:
                raise ArgumentError(command, f"unknown argument {prefix}{key}")
            continue
        if value is None and "null" not in types_of(spec):
            raise ArgumentError(command, f"{prefix}{key} must not be None; leave it out to use the default")
        _check_value(command, f"{prefix}{key}", spec, value)


def check(command: str, schema: dict, args: dict) -> None:
    if not isinstance(args, dict):
        raise ArgumentError(command, "arguments must be a dict")
    check_object(command, schema, args)
