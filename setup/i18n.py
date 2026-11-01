"""Interface text for translation.

  extract   collect the English interface strings into frontend/src/i18n/strings.json
  check     report each language table's coverage of strings.json and any {placeholder} mismatch

English is the source and the key: every table frontend/src/i18n/<code>.json maps an English string to its
translation. Regional variants (es-MX, fr-CA) keep only the words that differ from their parent.
"""

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SRC = ROOT / "frontend" / "src"
OUT = SRC / "i18n"
REGISTRY = ROOT / "native" / "crates" / "ffb-commands" / "src"
REGIONAL = {"es-MX", "fr-CA"}

CALLS = [
    re.compile(r"\bel\(\s*'[a-z0-9]+'\s*,\s*'[^']*'\s*,\s*'((?:[^'\\]|\\.)+)'"),
    re.compile(r"\bt\(\s*'((?:[^'\\]|\\.)+)'"),
    re.compile(r"\bfield\(\s*'((?:[^'\\]|\\.)+)'"),
    re.compile(r"\bchoices\(\s*'((?:[^'\\]|\\.)+)'"),
    re.compile(r"\bnote\(\s*'((?:[^'\\]|\\.)+)'"),
    re.compile(r"\[\s*'[a-z0-9_.-]+'\s*,\s*'((?:[^'\\]|\\.)+)'"),
    re.compile(r"\b(?:label|title|description)\s*:\s*'((?:[^'\\]|\\.)+)'"),
    re.compile(r"'(aria-label|placeholder|title)'\s*,\s*'((?:[^'\\]|\\.)+)'"),
    re.compile(r"\b(?:aria-label|title|placeholder)=\"([^\"$<>{}]+)\""),
    re.compile(r">([A-Z][^<>{}$`]{1,200})<"),
    re.compile(r"\b[cn]\(\s*'[a-z0-9_.]+'\s*,\s*'((?:[^'\\]|\\.)+)'"),
    re.compile(r"\bdef\(\s*'((?:[^'\\]|\\.)+)'"),
]
RUST_LABEL = re.compile(r'^\s*label:\s*"([^"]+)"', re.M)


def looks_like_text(s: str) -> bool:
    if not re.search(r"[A-Za-z]", s) or len(s) > 300:
        return False
    if re.fullmatch(r"[a-z0-9_.:/#-]+", s):
        return False
    if s.startswith(("http", "/", "--", ".", "#")) or "=>" in s:
        return False
    return True


def extract() -> int:
    found: set[str] = set()
    for f in sorted(SRC.rglob("*.ts")):
        if "i18n" in f.parts:
            continue
        text = f.read_text(encoding="utf-8")
        for rx in CALLS:
            for m in rx.finditer(text):
                s = m.group(m.lastindex or 1).replace("\\'", "'")
                if looks_like_text(s):
                    found.add(s.strip())
    if REGISTRY.is_dir():
        for f in sorted(REGISTRY.glob("*.rs")):
            for m in RUST_LABEL.finditer(f.read_text(encoding="utf-8")):
                found.add(m.group(1))
    found.discard("")
    OUT.mkdir(parents=True, exist_ok=True)
    strings = sorted(found, key=str.lower)
    (OUT / "strings.json").write_text(json.dumps(strings, ensure_ascii=False, indent=1) + "\n", encoding="utf-8")
    print(f"[formfabric] {len(strings)} interface strings in frontend/src/i18n/strings.json")
    return 0


def load_keys(path: Path) -> list[str]:
    data = json.loads(path.read_text(encoding="utf-8"))
    return list(data.keys()) if isinstance(data, dict) else list(data)


def check() -> int:
    source = OUT / "strings.json"
    if not source.is_file():
        print("[formfabric] strings.json is missing; run ./formfabric i18n extract")
        return 1
    strings = load_keys(source)
    bad = 0
    for path in sorted(OUT.glob("*.json")):
        code = path.stem
        if code == "strings":
            continue
        table = json.loads(path.read_text(encoding="utf-8"))
        if not isinstance(table, dict):
            print(f"  {code}: not an object of English string to translation")
            bad += 1
            continue
        missing = [s for s in strings if s not in table]
        for en, tr in table.items():
            if sorted(re.findall(r"\{\w+\}", en)) != sorted(re.findall(r"\{\w+\}", str(tr))):
                print(f"  {code}: placeholders differ in {en!r}")
                bad += 1
        if code in REGIONAL:
            print(f"  {code:6} regional overrides ({len(table)})")
            continue
        print(f"  {code:6} {len(strings) - len(missing)}/{len(strings)}")
        for s in missing[:5]:
            print(f"         missing {s!r}")
        if missing:
            bad += 1
    return 1 if bad else 0


if __name__ == "__main__":
    cmd = sys.argv[1] if len(sys.argv) > 1 else "check"
    sys.exit(extract() if cmd == "extract" else check() if cmd == "check" else 2)
