"""Interface text for translation.

  extract   print the English interface strings found in frontend/src (t('...'), t("..."), tn(n, '...', '...'))
  check     report parent-table keys that are missing and {placeholder} mismatches; exit 1 on problems

English is the source and the key. Each table frontend/src/ui/i18n/<code>.ts is a TypeScript object literal
mapping an English string to its translation. es-MX and fr-CA are regional delta tables over es and fr: they are
checked for placeholders only. t() looks a key up without its trailing '...' or ':' first, and so does check.
"""

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SRC = ROOT / "frontend" / "src"
TABLES = SRC / "ui" / "i18n"
REGIONAL = {"es-MX", "fr-CA"}
NOT_TABLES = {"langs"}

LITERAL = r"""(?:'((?:[^'\\\n]|\\.)*)'|"((?:[^"\\\n]|\\.)*)")"""
T_CALL = re.compile(r"\bt\(\s*" + LITERAL + r"\s*[,)]")
TN_CALL = re.compile(r"\btn\(\s*[^,()]+,\s*" + LITERAL + r"\s*,\s*" + LITERAL + r"\s*[,)]")
ENTRY = re.compile(r'^\s*"((?:[^"\\]|\\.)*)"\s*:\s*"((?:[^"\\]|\\.)*)",?\s*$')
PLACEHOLDER = re.compile(r"\{\w+\}")
ESCAPES = {"n": "\n", "t": "\t", "\\": "\\", "'": "'", '"': '"'}


def unescape(raw: str) -> str:
    return re.sub(r"\\(.)", lambda m: ESCAPES.get(m.group(1), m.group(1)), raw)


def has_text(s: str) -> bool:
    return bool(re.search(r"[A-Za-z]", s))


def extract_keys() -> list[str]:
    found: set[str] = set()
    for f in sorted(SRC.rglob("*.ts")):
        if TABLES in f.parents:
            continue
        text = f.read_text(encoding="utf-8")
        for m in T_CALL.finditer(text):
            found.add(unescape(m.group(1) if m.group(1) is not None else m.group(2)))
        for m in TN_CALL.finditer(text):
            for a, b in ((1, 2), (3, 4)):
                found.add(unescape(m.group(a) if m.group(a) is not None else m.group(b)))
    return sorted((k for k in found if k and has_text(k)), key=str.lower)


def load_table(path: Path) -> dict[str, str]:
    table: dict[str, str] = {}
    for line in path.read_text(encoding="utf-8").splitlines():
        m = ENTRY.match(line)
        if m:
            table[unescape(m.group(1))] = unescape(m.group(2))
    return table


def known(table: dict[str, str], key: str) -> bool:
    tail = "..." if key.endswith("...") else ":" if key.endswith(":") and not key.endswith("::") else ""
    base = key[: -len(tail)] if tail else key
    return base in table or (bool(tail) and key in table)


def placeholders(s: str) -> list[str]:
    return sorted(PLACEHOLDER.findall(s))


def extract() -> int:
    keys = extract_keys()
    for k in keys:
        print(k)
    print(f"[formfabric] {len(keys)} interface strings", file=sys.stderr)
    return 0


def check() -> int:
    keys = extract_keys()
    bad = 0
    for path in sorted(TABLES.glob("*.ts")):
        code = path.stem
        if code in NOT_TABLES:
            continue
        table = load_table(path)
        if not table:
            print(f"  {code}: no entries found")
            bad += 1
            continue
        for en, tr in table.items():
            if placeholders(en) != placeholders(tr):
                print(f"  {code}: placeholders differ in {en!r}")
                bad += 1
        if code in REGIONAL:
            print(f"  {code:7} regional overrides ({len(table)})")
            continue
        missing = [k for k in keys if not known(table, k)]
        print(f"  {code:7} {len(keys) - len(missing)}/{len(keys)}")
        for k in missing[:5]:
            print(f"          missing {k!r}")
        if missing:
            bad += 1
    return 1 if bad else 0


if __name__ == "__main__":
    cmd = sys.argv[1] if len(sys.argv) > 1 else "check"
    sys.exit(extract() if cmd == "extract" else check() if cmd == "check" else 2)
