#!/usr/bin/env python3
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

SITES = [
    ("VERSION", r"^\s*(\d+\.\d+\.\d+(?:-[0-9A-Za-z.]+)?)\s*$"),
    ("src-tauri/tauri.conf.json", "json:version"),
    ("src-tauri/Cargo.toml", r'^version = "([^"]+)"'),
    ("native/Cargo.toml", r'^version = "([^"]+)"'),
    ("backend/app/__init__.py", r'__version__ = "([^"]+)"'),
    ("frontend/package.json", "json:version"),
    ("frontend/src/version.ts", r"VERSION = '([^']+)'"),
    ("python/formfabric/__init__.py", r'__version__ = "([^"]+)"'),
    ("python/pyproject.toml", r'^version = "([^"]+)"'),
    ("README.md", r"badge/version-([0-9][0-9A-Za-z.]*?)-f97316"),
]


def read(path: Path, how: str) -> str | None:
    try:
        text = path.read_text(encoding="utf-8")
    except OSError:
        return None
    if how.startswith("json:"):
        try:
            return str(json.loads(text).get(how[5:]))
        except ValueError:
            return None
    found = re.search(how, text, re.M)
    return found.group(1) if found else None


LOCKS = ["native/Cargo.lock", "src-tauri/Cargo.lock"]


def lock_entries(path: Path) -> list[tuple[str, str]] | None:
    try:
        text = path.read_text(encoding="utf-8")
    except OSError:
        return None
    found = re.findall(r'^name = "(ffb-[a-z0-9-]+|formfabric)"\nversion = "([^"]+)"', text, re.M)
    return found


def changelog_has(version: str) -> bool:
    try:
        text = (ROOT / "CHANGELOG.md").read_text(encoding="utf-8")
    except OSError:
        return False
    return re.search(rf"^## v?{re.escape(version)}\b", text, re.M) is not None


def main(argv: list[str]) -> int:
    command = argv[0] if argv else "check"
    if command not in ("check", "sites", "print"):
        print("usage: python3 setup/version.py [check|sites|print]")
        return 2
    values = [(name, read(ROOT / name, how)) for name, how in SITES]
    if command == "sites":
        for name, value in values:
            print(f"{name:34} {value if value is not None else 'MISSING'}")
        for lock in LOCKS:
            entries = lock_entries(ROOT / lock)
            print(f"{lock:34} {', '.join(sorted({v for _, v in entries})) if entries else 'MISSING'}")
        return 0
    anchor = read(ROOT / "src-tauri/tauri.conf.json", "json:version")
    if command == "print":
        print(anchor or "")
        return 0 if anchor else 1
    bad = [(n, v) for n, v in values if v != anchor]
    for name, value in bad:
        print(f"[formfabric] {name}: {value if value is not None else 'missing'} (expected {anchor})")
    for lock in LOCKS:
        entries = lock_entries(ROOT / lock)
        if entries is None:
            print(f"[formfabric] {lock}: missing")
            bad.append((lock, None))
            continue
        for crate, value in entries:
            if value != anchor:
                print(f"[formfabric] {lock}: {crate} {value} (expected {anchor})")
                bad.append((f"{lock}:{crate}", value))
    if anchor and not changelog_has(anchor):
        print(f"[formfabric] CHANGELOG.md has no heading for {anchor}")
        bad.append(("CHANGELOG.md", None))
    if bad:
        return 1
    print(f"[formfabric] one version, {anchor}, across {len(values)} sites and the changelog")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
