from __future__ import annotations

import argparse
import json
import sys

from . import commands, connect
from .commands import write_snapshot
from .errors import FormfabricError


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(prog="python -m formfabric", description="formfabric Python API")
    parser.add_argument("--backend", choices=["auto", "engine", "http"], default="auto")
    parser.add_argument("--url")
    sub = parser.add_subparsers(dest="cmd", required=True)
    sub.add_parser("commands", help="list the commands of the connected backend")
    snap = sub.add_parser("snapshot", help="write the connected backend's catalogue into the package (commands.py reads it)")
    snap.add_argument("--print", action="store_true", dest="show")
    call = sub.add_parser("call", help="run one command: call sample.load name=tshirt force=true")
    call.add_argument("command")
    call.add_argument("pairs", nargs="*", help="key=value, the value as JSON when it parses, else text")
    args = parser.parse_args(argv)
    try:
        client = connect(args.backend, args.url)
        if args.cmd == "commands":
            for spec in client.catalogue()["commands"]:
                print(f"{spec['id']:32} {spec['group']:10} {spec['label']}")
            return 0
        if args.cmd == "snapshot":
            catalogue = client.catalogue()
            if args.show:
                json.dump(catalogue, sys.stdout)
            else:
                print(write_snapshot(catalogue), len(catalogue["commands"]), "commands")
            return 0
        values = {}
        for pair in args.pairs:
            key, _, raw = pair.partition("=")
            try:
                values[key] = json.loads(raw)
            except ValueError:
                values[key] = raw
        print(json.dumps(client.call(args.command, **values), indent=2))
        return 0
    except FormfabricError as exc:
        print(f"formfabric: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
