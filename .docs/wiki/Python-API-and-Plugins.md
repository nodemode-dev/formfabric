# Python API and plugins

Script formfabric from Python in two ways: the `formfabric` package for scripts and notebooks, and plugins that run inside the service.

## Where it lives

| Path | Holds |
| --- | --- |
| `python/formfabric/` | the package: `backend.py`, `client.py`, `commands.py`, `model.py`, `gof.py`, `schema.py`, `errors.py`, `__main__.py`, `catalogue.json` |
| `python/pyproject.toml`, `python/README.md` | packaging (standard library only, Python 3.10 or later) and the full guide |
| `backend/app/services/scripting.py`, `routers/plugins.py` | the plugin runner and `POST /api/plugins/run` |
| `backend/plugins/` | bundled plugins; `drape_and_export.py` is the sample |

## The package

```python
import formfabric as ff
from formfabric import commands as c

ff.use(ff.connect())
g = ff.Garment.sample("tshirt")
g.sim.drape(60)
g.export_glb("/tmp/tshirt.glb")
c.sim_step(frames=10)
```

Run scripts with `./formfabric py script.py`. `connect()` chooses the `http` backend when `FFB_ENGINE_URL` is set (the window's terminal sets it) or a url or token is given, otherwise the in-process `engine` backend over the `ffb_engine` module.

- `formfabric.commands` has one function per command, named after the id with dots as underscores. Arguments are keyword only and checked against the schema before sending (`ArgumentError`); engine failures raise `CommandError` with the engine's `code`. Docstrings come from the catalogue.
- The hand API: `Garment.new`, `sample`, `open`, `save`, `rectangle`, `draw`, `export`, `export_glb`; `Garment.sim` (`reset`, `step`, `start`, `stop`, `state`, `settings`, `preset`, `drape`, `wait`); `Garment.link(folder, every_frames)` for the Live Link; `Panel` helpers; `gof.send`.
- `catalogue.json` is a snapshot of the engine catalogue. After the last command change refresh it with `./formfabric py -m formfabric snapshot`, then run `./formfabric agents sync`.

## The HTTP backend and tokens

Only FormFabric's own window can change things through the service. The client sends `FFB_WINDOW_TOKEN` (or `token=`, 32 or more characters, the value the service was started with) in `x-ffb-window`. An agent's token opens only `/api/mcp`, never the window's routes, so a script run by an agent or in the window's terminal should use the in-process backend. The host must be 127.0.0.1 or localhost.

## Plugins

A plugin is a `.py` file with a top-level `run(ffb, args)` and an optional `PARAMS` dict, kept in `backend/plugins`, `~/.formfabric/plugins` or `FFB_PLUGIN_DIR`. `PARAMS` defaults are applied and values are checked before `run` (failures are `bad_args`). Inside a plugin `ffb.call("sim.step", frames=10)` runs a command and raises `ScriptError` (with `code`) on failure; `ffb.try_call` returns the failure instead, and `ffb.commands` lists names. Run one with `POST /api/plugins/run` and `{"name": "drape_and_export", "args": {"sample": "tshirt", "frames": 60, "format": "glb", "path": "/tmp/out.glb"}}`. That plugin starts from a new project, so save yours first. An unbuilt engine answers 503. Packaged apps bundle `plugins/`.

## Contract

The package gets every command from the catalogue, so it must not hand-write a command wrapper. Keep it standard library only. A script must not use the `/api/mcp` consent flow; the token is the authorisation.
