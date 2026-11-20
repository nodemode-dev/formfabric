#!/usr/bin/env python3
import glob
import hashlib
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
    if check:
        for problem in dock_problems():
            say(problem)
        if dock_problems():
            return 1
    try:
        cat = catalogue()
        block = render(cat)
        current = MODELING.read_text(encoding="utf-8")
        updated = splice(current, block)
        manifest = manifest_text(cat)
    except NoEngine as err:
        say(str(err))
        return 1
    if check:
        drift = current != updated or not mirrored_equal() or not MANIFEST.is_file() or MANIFEST.read_text(encoding="utf-8") != manifest
        say("the modeling skill's tool reference has drifted; run ./formfabric agents sync" if drift else "tool reference up to date")
        return 1 if drift else 0
    MODELING.write_text(updated, encoding="utf-8")
    if MIRROR.exists():
        shutil.rmtree(MIRROR)
    MIRROR.parent.mkdir(parents=True, exist_ok=True)
    shutil.copytree(SKILLS, MIRROR)
    MANIFEST.parent.mkdir(parents=True, exist_ok=True)
    MANIFEST.write_text(manifest, encoding="utf-8")
    say(f"wrote {MANIFEST.relative_to(ROOT)}")
    say(f"wrote the tool reference in {MODELING.relative_to(ROOT)} and mirrored the skills to {MIRROR.relative_to(ROOT)}")
    return 0


MANIFEST = ROOT / "agents" / "manifest.json"
DOCK = {"present": True, "positions": ["bottom", "right"], "tabs": ["chat", "terminal", "claude-code", "codex", "cursor", "antigravity"]}
CLIENTS = ("claude", "codex", "cursor", "antigravity")
DOCK_DIR = ROOT / "frontend" / "src" / "dock"
DOCK_MANIFEST = DOCK_DIR / "MANIFEST.json"
DOCK_VERSION = "2.2.1"


def _dock_files() -> dict[str, str]:
    return {p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(DOCK_DIR.iterdir()) if p.is_file() and p.name != DOCK_MANIFEST.name}


def dock_problems() -> list[str]:
    try:
        held = json.loads(DOCK_MANIFEST.read_text(encoding="utf-8"))
    except (OSError, ValueError):
        return [f"{DOCK_MANIFEST.relative_to(ROOT)}: missing or not JSON"]
    rel = DOCK_DIR.relative_to(ROOT).as_posix()
    problems = [] if held.get("version") == DOCK_VERSION else [f"{rel}/MANIFEST.json: version {held.get('version')}, this app expects {DOCK_VERSION}"]
    kept, now = held.get("files", {}), _dock_files()
    return problems + [f"{rel}/{name}: does not match MANIFEST.json (the shared dock module changes only as a whole new version)" for name in sorted(set(kept) | set(now)) if kept.get(name) != now.get(name)]


def manifest_text(cat: dict) -> str:
    tools = cat["commands"]
    names = [c["name"] for c in tools]
    read_only = [c["name"] for c in tools if (c.get("annotations") or {}).get("readOnlyHint") is True]
    skills = []
    for folder in sorted(p for p in SKILLS.iterdir() if (p / "SKILL.md").is_file()):
        subs = sorted(s.parent.name for s in folder.rglob("SUB-SKILL.md") if s.parent != folder)
        skills.append({"name": folder.name, "sub_skills": subs})
    data = {
        "format": "form-agent-manifest",
        "version": 1,
        "app": "formfabric",
        "mcp": {"tools": names, "read_only": read_only, "approval_gated": [n for n in names if n not in read_only], "transports": ["stdio", "http"]},
        "skills": skills,
        "dock": DOCK,
        "extensions": None,
    }
    return json.dumps(data, indent=2, ensure_ascii=False) + "\n"


def check() -> int:
    problems = dock_problems()
    try:
        cat = catalogue()
        if MODELING.read_text(encoding="utf-8") != splice(MODELING.read_text(encoding="utf-8"), render(cat)):
            problems.append(f"{MODELING.relative_to(ROOT)} has drifted from the engine catalogue; run ./formfabric agents sync")
        if not MANIFEST.is_file() or MANIFEST.read_text(encoding="utf-8") != manifest_text(cat):
            problems.append(f"{MANIFEST.relative_to(ROOT)} is missing or out of date; run ./formfabric agents sync")
    except NoEngine as err:
        problems.append(str(err))
    if not mirrored_equal():
        problems.append(f"{MIRROR.relative_to(ROOT)} does not mirror {SKILLS.relative_to(ROOT)}; run ./formfabric agents sync")
    for folder in (p for p in SKILLS.iterdir() if p.is_dir()):
        text = (folder / "SKILL.md").read_text(encoding="utf-8") if (folder / "SKILL.md").is_file() else ""
        if not text.startswith("---") or f"name: {folder.name}" not in text or "description:" not in text:
            problems.append(f"{folder.relative_to(ROOT)}/SKILL.md lacks front matter naming {folder.name} with a description")
    for rel, key in ((".mcp.json", "mcpServers"), (".cursor/mcp.json", "mcpServers")):
        try:
            server = json.loads((ROOT / rel).read_text(encoding="utf-8"))[key]["formfabric"]
            if "formfabric" not in server.get("command", "") or server.get("args") != ["mcp"]:
                problems.append(f"{rel} does not start the formfabric launcher with mcp")
        except (OSError, ValueError, KeyError, TypeError):
            problems.append(f"{rel} has no formfabric MCP server")
    if "[mcp_servers.formfabric]" not in (ROOT / ".codex" / "config.toml").read_text(encoding="utf-8"):
        problems.append(".codex/config.toml has no formfabric MCP server")
    for p in problems:
        say(p)
    if not problems:
        say("clean: tool reference, skill mirror, skill front matter, MCP configs and agents/manifest.json agree")
    return 1 if problems else 0


def install_antigravity(command: str, dry: bool) -> int:
    cfg = Path.home() / ".gemini" / "config" / "mcp_config.json"
    try:
        data = json.loads(cfg.read_text(encoding="utf-8") or "{}") if cfg.exists() else {}
    except (OSError, ValueError):
        data = None
    if not isinstance(data, dict) or not isinstance(data.get("mcpServers", {}), dict):
        say(f"{cfg} is not an MCP servers file formfabric can read; left untouched")
        return 1
    entry = {"command": command, "args": ["mcp"]}
    have = data.get("mcpServers", {}).get("formfabric")
    if have == entry:
        say(f"{cfg} already has formfabric")
    elif have is not None and not str((have or {}).get("command", "")).endswith("formfabric"):
        say(f"{cfg} has a server named formfabric that formfabric did not write; left untouched")
    else:
        say(("would write" if dry else "writing") + f" formfabric into {cfg}")
        if not dry:
            cfg.parent.mkdir(parents=True, exist_ok=True)
            if cfg.exists():
                shutil.copy2(cfg, cfg.with_suffix(".json.bak"))
            data.setdefault("mcpServers", {})["formfabric"] = entry
            cfg.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")
    return 0


def install(dry: bool, targets: list[str]) -> int:
    command = os.environ.get("FORMFABRIC_LAUNCHER") or str(ROOT / "formfabric")
    targets = targets or list(CLIENTS)
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
        block = f'\n[mcp_servers.formfabric]\ncommand = {json.dumps(command)}\nargs = ["mcp"]\nstartup_timeout_sec = 90\ntool_timeout_sec = 600\n'
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
        try:
            data = json.loads(cfg.read_text(encoding="utf-8")) if cfg.exists() else {}
        except (OSError, ValueError):
            say(f"{cfg} is not readable JSON; left untouched")
            return 1
        if not isinstance(data, dict):
            say(f"{cfg} is not a JSON object; left untouched")
            return 1
        servers = data.setdefault("mcpServers", {})
        if not isinstance(servers, dict):
            say(f"{cfg} has a malformed mcpServers; left untouched")
            return 1
        if servers.get("formfabric", {}).get("command") == command:
            say(f"{cfg} already has formfabric")
        elif "formfabric" in servers and not str(servers["formfabric"].get("command", "")).endswith("formfabric"):
            say(f"{cfg} has a server named formfabric that formfabric did not write; left untouched")
        else:
            servers["formfabric"] = {"command": command, "args": ["mcp"]}
            say(("would write" if dry else "writing") + f" formfabric into {cfg}")
            if not dry:
                cfg.parent.mkdir(parents=True, exist_ok=True)
                if cfg.exists():
                    shutil.copy2(cfg, cfg.with_suffix(".json.bak"))
                cfg.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")
    if "antigravity" in targets:
        return install_antigravity(command, dry)
    return 0


def main(argv: list[str]) -> int:
    if not argv or argv[0] not in ("sync", "check", "install") or any(a not in (*CLIENTS, "--dry-run", "--check") for a in argv[1:]):
        print("usage: ./formfabric agents sync [--check] | check | install [--dry-run] [claude|codex|cursor|antigravity ...]")
        return 2
    if argv[0] == "check":
        return check()
    if argv[0] == "sync":
        return sync("--check" in argv[1:])
    rest = [a for a in argv[1:] if a not in ("--dry-run",)]
    return install("--dry-run" in argv[1:], rest)


if __name__ == "__main__":
    os.chdir(ROOT)
    sys.exit(main(sys.argv[1:]))
