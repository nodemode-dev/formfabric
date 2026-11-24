# Testing formfabric

formfabric's tests run the app and check what a user or a calling program would notice. This page says how to run them, where they live and what a test must do to be accepted.

## Running the tests

```sh
./formfabric test                       # every layer
./formfabric test engine                # one layer: unit or engine
./formfabric test engine -k seal        # the rest of the line goes to the runner
```

On Windows use `formfabric.bat test`. Run `./formfabric install` first. The engine layer needs pytest and httpx in the backend venv; when they are missing the runner says so and names the command:

```sh
backend/venv/bin/python -m pip install pytest httpx
```

An unknown layer exits with code 2 and a sentence that names the layers.

## The layers

| Layer | Tool | Where | What it runs against |
| --- | --- | --- | --- |
| unit | vitest, happy-dom | `frontend/tests/unit/` | exported interface functions and stores, no browser |
| engine | pytest | `backend/tests/` | the real FastAPI app in process, with a temporary data folder, a temporary home folder and stand-ins for the agent programs |

Not covered yet: the native Rust workspace in `native/` has no tests in this version, and neither has the desktop shell in `src-tauri/`.

## Where a lane's tests live

The tests are grouped by area, one folder for each lane: `backend/tests/<lane>/` for the engine layer and `frontend/tests/unit/<lane>/` for the unit layer. A unit test file is named `*.test.ts`, which is what `frontend/vitest.config.ts` collects. A pytest file is named `test_*.py`.

## What the shared base gives a test

`backend/tests/conftest.py` sets the data folder, the home folder and the port before the app is imported, removes every `FFB_` and `FORMFABRIC_` variable from the environment, and puts stand-ins for the agent programs first on the path, so no test starts a real agent program or reads a real settings file. It provides `app` (the FastAPI app), `client` (a client that holds the window key), `bare` (a client without the key) and pytest's `tmp_path`. The run ends with a line that says how many times an agent stand-in was started.

## The five rules

1. The test runs the app through a public surface: an HTTP route, a command through the command runner, an MCP tool, a worker request, an exported interface function, or the visible interface in a browser.
2. It checks a result a user or a calling program would notice: geometry, a number, a saved or exported file, a refusal, an undo, something on screen.
3. Its expected value comes from outside the code under test: mathematics, the rules of a file format checked by an independent reader, the manual and the format documents in `docs/`, an invariant (undo then redo, save then open, export then import), two independent paths that must agree, or a fixture that states its own content.
4. It can fail. Change the expected value once, see the test fail, and change it back.
5. Its name says the behaviour in plain words.

Not accepted:

- reading source files to check text, spelling, comments or formatting;
- checks that a file, function, class or route merely exists;
- a check of the status code alone, with nothing checked in the answer;
- snapshot files made by running the code and saving what came out;
- replacing the thing under test with a stand-in and testing the stand-in;
- a skip or an expected failure without a written reason.

## When a test shows the app is wrong

Do not bend the test. Keep it and mark it as a strict expected failure with a reason that starts with `DEFECT:` and says what the app does, what it should do and where that is stated. `git grep -n "DEFECT:" backend/tests frontend/tests` lists the known ones. When the app is fixed, the strict mark makes the test fail until the mark is removed, so a fix cannot go unnoticed.

## Changing behaviour

A change in behaviour updates its tests in the same pull request. Tests are added when the owner asks. New test data is made by hand or by formfabric itself. No other garment or pattern product is named in a test, and only file-format names appear.

## CI

Every pull request runs one job, `test`, on Linux. It makes the backend venv from `backend/requirements.txt`, adds pytest and httpx, installs the frontend from its lock file, then runs the two layers one step each, so the step that fails names the layer. The job does not build the Rust engine module. The tests do not run on macOS or Windows in CI.
