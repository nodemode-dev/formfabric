"""Drape a sample and export it
Drafts a sample garment on the default body, drapes it for the given number of frames and writes the draped garment to a 3D file. Starts from a new project, so save yours first.
"""

import os

PARAMS = {
    "sample": {"type": "choice", "label": "Sample", "options": ["tshirt", "skirt", "dress"], "default": "tshirt"},
    "frames": {"type": "integer", "label": "Drape frames", "default": 60, "min": 1, "max": 600, "step": 10},
    "format": {"type": "choice", "label": "Format", "options": ["glb", "gltf", "obj", "fbx", "usd"], "default": "glb"},
    "path": {"type": "text", "label": "Output file (empty for the exports folder)", "default": ""},
}

EXTENSIONS = {"usd": "usda"}


def run(ffb, args):
    sample, frames, kind = args["sample"], args["frames"], args["format"]
    path = args.get("path") or ""
    if not path:
        root = os.environ.get("FORMFABRIC_DATA_DIR") or os.path.join(os.path.expanduser("~"), "Formfabric")
        os.makedirs(os.path.join(root, "exports"), exist_ok=True)
        path = os.path.join(root, "exports", f"{sample}_drape.{EXTENSIONS.get(kind, kind)}")
    ffb.call("file.new")
    loaded = ffb.call("sample.load", name=sample, force=True)
    ffb.call("sim.reset")
    ffb.call("sim.step", frames=frames)
    state = ffb.call("sim.state")
    written = ffb.call("export.run", format=kind, path=path)
    print(f"{sample}: {len(loaded.get('panels', []))} panels, {state.get('vertices')} vertices, frame {state.get('frame')}, wrote {written.get('path')}")
    return {"path": written.get("path"), "frame": state.get("frame"), "vertices": state.get("vertices"), "files": written.get("files")}
