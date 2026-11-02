#!/usr/bin/env python3
import glob
import json
import os
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SKILLS = ROOT / ".claude" / "skills"
MIRROR = ROOT / ".agents" / "skills"
MODELING = SKILLS / "formfabric-modeling" / "SKILL.md"
BEGIN = "[ffb-tools:begin]"
END = "[ffb-tools:end]"
GROUP_TITLES = {
    "system": "Engine",
    "file": "Files",
    "history": "History",
    "view": "Viewing",
    "pattern": "Pattern",
    "sew": "Sewing",
    "sim": "Simulation",
    "body": "Bodies",
    "fabric": "Fabrics",
    "anim": "Animation",
    "export": "Export",
}


class NoEngine(Exception):
    pass


def say(text: str) -> None:
    print(f"[formfabric] {text}")


def from_module() -> dict | None:
    if os.environ.get("FFB_CATALOGUE_BIN"):
        return None
    for site in glob.glob(str(ROOT / "backend" / "venv" / "lib" / "python*" / "site-packages")) + [str(ROOT / "backend" / "venv" / "Lib" / "site-packages")]:
        if os.path.isdir(site) and site not in sys.path:
            sys.path.insert(0, site)
    try:
        import ffb_engine
    except ImportError:
        return None
    return json.loads(ffb_engine.catalogue())


def from_binary() -> dict | None:
    chosen = os.environ.get("FFB_CATALOGUE_BIN")
    if chosen:
        return json.loads(subprocess.run([chosen], check=True, capture_output=True, text=True).stdout)
    names = ("ffb-catalogue", "ffb-catalogue.exe")
    found = [Path(p) for n in names for p in glob.glob(str(ROOT / "native" / "target" / "*" / n))]
    found += [Path(p) for n in names for p in glob.glob(str(ROOT / "native" / "target" / "*" / "*" / n))]
    if not found:
        return None
    newest = max(found, key=lambda p: p.stat().st_mtime)
    out = subprocess.run([str(newest)], check=True, capture_output=True, text=True).stdout
    return json.loads(out)


def catalogue() -> dict:
    for source in (from_module, from_binary):
        try:
            cat = source()
        except (OSError, subprocess.CalledProcessError, json.JSONDecodeError) as err:
            raise NoEngine(f"the engine catalogue could not be read ({err})")
        if cat:
            return cat
    raise NoEngine("the engine is not built: no ffb_engine in backend/venv and no native/target/*/ffb-catalogue; run ./formfabric engine build")


def describe(name: str, spec: dict, required: set) -> str:
    kind = spec.get("type", "value")
    if kind == "array" and isinstance(spec.get("items"), dict):
        kind = f"array of {spec['items'].get('type', 'value')}"
    parts = [f"- `{name}`: {kind}"]
    if "enum" in spec:
        parts.append(" (" + " | ".join(str(v) for v in spec["enum"]) + ")")
    if name in required:
        parts.append("; required")
    if "default" in spec:
        parts.append(f"; default {json.dumps(spec['default'])}")
    for lo, hi in (("minimum", "maximum"), ("exclusiveMinimum", "maximum")):
        if lo in spec or hi in spec:
            parts.append(f"; range {spec.get(lo, '-inf')} to {spec.get(hi, 'inf')}")
            break
    if spec.get("description"):
        parts.append(f"; {spec['description']}")
    if spec.get("type") == "object" and isinstance(spec.get("properties"), dict):
        parts.append("; fields " + ", ".join(spec["properties"].keys()))
    return "".join(parts)


def render(cat: dict) -> str:
    lines = [
        f"Generated from the engine's command registry (formfabric {cat['version']}) by `./formfabric agents sync`; do not edit between the markers by hand.",
        "",
    ]
    groups: dict[str, list[dict]] = {}
    for c in cat["commands"]:
        groups.setdefault(c["group"], []).append(c)
    for g, cmds in groups.items():
        lines.append(f"- {GROUP_TITLES.get(g, g.title())}: " + ", ".join(f"`{c['name']}`" for c in cmds))
    lines.append("")
    for c in cat["commands"]:
        lines += [f"### {c['name']}", "", c["description"], ""]
        schema = c["inputSchema"]
        props = schema.get("properties") or {}
        required = set(schema.get("required") or [])
        if props:
            lines += [describe(k, v, required) for k, v in props.items()]
        else:
            lines.append("- no parameters")
        hints = c["annotations"]
        notes = []
        if hints.get("readOnlyHint"):
            notes.append("read-only")
        if hints.get("destructiveHint"):
            notes.append("removes or replaces (undoable)")
        if hints.get("idempotentHint") and not hints.get("readOnlyHint"):
            notes.append("safe to repeat")
        if c.get("keys"):
            notes.append("in the window: " + ", ".join(c["keys"]))
        if notes:
            lines += ["", "Note: " + "; ".join(notes) + "."]
        lines.append("")
    return "\n".join(lines).rstrip()


def splice(text: str, block: str) -> str:
    a = text.find(BEGIN)
    b = text.find(END)
    if a < 0 or b < a:
        raise NoEngine(f"{MODELING.relative_to(ROOT)} is missing the {BEGIN} and {END} markers")
    return text[: a + len(BEGIN)] + "\n" + block + "\n" + text[b:]


def mirrored_equal() -> bool:
    if not MIRROR.exists():
        return False
    for f in SKILLS.rglob("*"):
        if f.is_file():
            m = MIRROR / f.relative_to(SKILLS)
            if not m.is_file() or m.read_bytes() != f.read_bytes():
                return False
    return True


def sync(check: bool) -> int:
    try:
        block = render(catalogue())
        current = MODELING.read_text(encoding="utf-8")
        updated = splice(current, block)
    except NoEngine as err:
        say(str(err))
        return 1
    if check:
        drift = current != updated or not mirrored_equal()
        say("the modeling skill's tool reference has drifted; run ./formfabric agents sync" if drift else "tool reference up to date")
        return 1 if drift else 0
    MODELING.write_text(updated, encoding="utf-8")
    if MIRROR.exists():
        shutil.rmtree(MIRROR)
    MIRROR.parent.mkdir(parents=True, exist_ok=True)
    shutil.copytree(SKILLS, MIRROR)
    say(f"wrote the tool reference in {MODELING.relative_to(ROOT)} and mirrored the skills to {MIRROR.relative_to(ROOT)}")
    return 0


def install(dry: bool, targets: list[str]) -> int:
    command = os.environ.get("FORMFABRIC_LAUNCHER") or str(ROOT / "formfabric")
    targets = targets or ["claude", "codex", "cursor"]
    if "claude" in targets:
        if shutil.which("claude"):
            args = ["claude", "mcp", "add", "--scope", "user", "formfabric", "--", command, "mcp"]
            say(("would run: " if dry else "running: ") + " ".join(args))
            if not dry:
                subprocess.run(args, check=False)
            skills = Path.home() / ".claude" / "skills" / "formfabric-modeling"
            say(("would copy" if dry else "copying") + f" the modeling skill to {skills}")
            if not dry:
                if skills.exists():
                    shutil.rmtree(skills)
                shutil.copytree(SKILLS / "formfabric-modeling", skills)
        else:
            say("Claude Code's claude command was not found; the project's .mcp.json still works inside this folder")
    if "codex" in targets:
        cfg = Path.home() / ".codex" / "config.toml"
        block = f'\n[mcp_servers.formfabric]\ncommand = "{command}"\nargs = ["mcp"]\nstartup_timeout_sec = 90\ntool_timeout_sec = 600\n'
        existing = cfg.read_text(encoding="utf-8") if cfg.exists() else ""
        if "[mcp_servers.formfabric]" in existing:
            say(f"{cfg} already has formfabric")
        else:
            say(("would append" if dry else "appending") + f" formfabric to {cfg}")
            if not dry:
                cfg.parent.mkdir(parents=True, exist_ok=True)
                if cfg.exists():
                    shutil.copy2(cfg, cfg.with_suffix(".toml.bak"))
                cfg.write_text(existing + block, encoding="utf-8")
    if "cursor" in targets:
        cfg = Path.home() / ".cursor" / "mcp.json"
        data = json.loads(cfg.read_text(encoding="utf-8")) if cfg.exists() else {}
        servers = data.setdefault("mcpServers", {})
        if servers.get("formfabric", {}).get("command") == command:
            say(f"{cfg} already has formfabric")
        else:
            servers["formfabric"] = {"command": command, "args": ["mcp"]}
            say(("would write" if dry else "writing") + f" formfabric into {cfg}")
            if not dry:
                cfg.parent.mkdir(parents=True, exist_ok=True)
                if cfg.exists():
                    shutil.copy2(cfg, cfg.with_suffix(".json.bak"))
                cfg.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")
    return 0


def main(argv: list[str]) -> int:
    if not argv or argv[0] not in ("sync", "install"):
        print("usage: ./formfabric agents sync [--check] | install [--dry-run] [claude|codex|cursor ...]")
        return 2
    if argv[0] == "sync":
        return sync("--check" in argv[1:])
    rest = [a for a in argv[1:] if a not in ("--dry-run",)]
    return install("--dry-run" in argv[1:], rest)


if __name__ == "__main__":
    os.chdir(ROOT)
    sys.exit(main(sys.argv[1:]))
