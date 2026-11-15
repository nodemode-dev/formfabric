# formfabric Python API

A pure Python package (standard library only, Apache-2.0) that drives formfabric from scripts, notebooks and plugins. It has two backends behind one client:

| Backend | What it talks to | When |
| --- | --- | --- |
| `engine` (in-process) | the `ffb_engine` module, its own private session | `ffb_engine` is importable (built by `./formfabric engine build`); no service needed |
| `http` | a running formfabric service, `POST /api/engine/dispatch` | you want to script the document the window shows |

`connect()` picks `http` when `FFB_ENGINE_URL` is set (the app's terminal sets it) or a url or token is given, otherwise `engine`, falling back to `http` when the module is missing.

## Run it

```
./formfabric py my_script.py          # the backend venv's Python with python/ on the path
./formfabric py -m formfabric commands
```

or `pip install ./python` into any Python 3.10 or newer, with `ffb_engine` installed for the in-process backend.

## The two minutes version

```python
import formfabric as ff
from formfabric import commands as c

client = ff.connect()                       # or ff.connect("http", url="http://127.0.0.1:8280", token="...")
ff.use(client)                              # default client for the commands module

g = ff.Garment.sample("tshirt")             # drafts the sample on the default body
g.sim.drape(60)                             # reset, 60 frames, returns the simulation state
g.export_glb("/tmp/tshirt.glb")

c.sim_step(frames=10)                       # one function per command, arguments checked first
c.pattern_rectangle(width=400, height=300, name="swatch")
```

## Commands

`formfabric.commands` has one function per catalogue command, named after the command id with dots as underscores (`sample.load` is `sample_load`, also `commands.sample.load`). Each function:

- takes keyword arguments only, named as in the command schema (a name that is a Python keyword gets a trailing underscore);
- checks them against the command's JSON schema before sending (required arguments, unknown names, types, enums, minimum and maximum, array sizes) and raises `ArgumentError`;
- returns the command's result, or raises `CommandError` carrying the engine's `code` and message;
- has the command's description and argument list as its docstring (`help(c.sim_step)`);
- accepts `client=` to use a client other than the default.

The functions are generated from `formfabric/catalogue.json`, a snapshot of the engine's catalogue shipped with the package. When the commands change, refresh the snapshot against a current engine:

```
./formfabric py -m formfabric snapshot
```

A client built on a newer engine also exposes the live catalogue: `client.bind()` returns a `Commands` object with a function for every command of that backend, and `client.call("link.start", folder=...)` works for any command name whether or not the snapshot has it.

## The hand API

| Name | Does |
| --- | --- |
| `Garment.new()`, `Garment.sample(name, body=None)`, `Garment.open(path)`, `save(path)` | start, load and save a project |
| `Garment.panels`, `panel(id_or_name)`, `rectangle(width, height, ...)`, `draw(points, ...)` | read and add panels |
| `Garment.sim` (`Sim`) | `reset`, `step(frames)`, `start`, `stop`, `state`, `settings(**)`, `preset(name)`, `drape(frames)`, `wait(frame)` |
| `Garment.export(path, format)`, `export_glb(path)` | export the draped garment |
| `Garment.link(folder, every_frames)` (`Link`) | start the Live Link; `Link.stop()`, `state()`, usable as a context manager |
| `Panel.rename`, `measure`, `positions`, `duplicate`, `delete` | per panel helpers |
| `gof.send(name, target, folder)` | write a GoF-compatible bundle (see `docs/integration/gof-handoff.md`) |

## The HTTP backend and the window token

The service refuses window routes unless the caller proves it is the window or an agent the window allowed. The Python client sends one of:

1. the token you pass as `token=`, or `FFB_WINDOW_TOKEN`, in the `x-ffb-window` header. It must be the value the service was started with (`./formfabric boot` generates one and the desktop shell makes one per launch; the service removes it from its own environment, so export the same value yourself when you start the service by hand). Tokens under 32 characters are refused by the service and by this client;
2. `FFB_INAPP_TOKEN` in the `x-formfabric-inapp` header. The service gives this to the Claude Code it starts inside the window, so a script run by that Claude Code works without any setup.

The window's own terminal deliberately does not receive either token, so a script run by hand in that terminal should use the in-process backend or be given a token. This client does not use the outside-agent consent flow of `/api/mcp` (that gate belongs to MCP over HTTP); the token is the whole authorisation, and the host must be 127.0.0.1 or localhost on the service's port.

## Plugins

Python plugins are files with a top-level `run(ffb, args)` and an optional `PARAMS` dict, kept in `backend/plugins`, `~/.formfabric/plugins` or `FFB_PLUGIN_DIR`. Inside a plugin `ffb.call("sim.step", frames=10)` runs a command and raises on failure. `backend/plugins/drape_and_export.py` drafts a sample, drapes it and exports a GLB. They run from `POST /api/plugins/run` with `{"name": ..., "args": {...}}`; defaults and ranges from `PARAMS` are applied and checked first.

## Errors

| Error | Meaning |
| --- | --- |
| `ArgumentError` | the arguments failed the schema; nothing was sent |
| `CommandError` | the engine refused or failed; `.code` and `.message` are its own |
| `BackendError` | no engine module, service unreachable, token refused, engine not built in the service |

## Check

`check_family.py`, kept with the build tools outside this tree, drives a sample drape through both backends, the Live Link, GoF and the sample plugin.
