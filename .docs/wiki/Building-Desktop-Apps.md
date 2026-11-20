# Building desktop apps

The installable app is a Tauri 2 shell around a frozen copy of the Python service and the Rust engine module. Developers build it with `build_app`; releases for macOS, Windows and Linux come from formcompiler, and formportal lists the result.

## Where it lives

| Path | Holds |
| --- | --- |
| `src-tauri/` | the Tauri shell: `tauri.conf.json` (identifier `ai.nodemode.formfabric`, `.ffbrc` file association, sidecar, updater block), `capabilities/`, `entitlements.plist`, `src/` |
| `setup/freeze_backend` (and `.bat`) | PyInstaller freeze of the service, bundling `backend/plugins` and `.claude/skills` |
| `setup/desktop_build`, `setup/desktop_dev`, `setup/build_frontend` | the build steps |
| `setup/version.py`, `VERSION` | the version sites check |
| `build_app`, `build_app.bat` | the start-to-finish build |
| `.github/workflows/ci.yml` | the check workflow formcompiler requires |
| `docs/integration/` | the registry, catalogue and product drafts for the other repositories |

## build_app

```bash
./build_app --dry-run     # checks files, executable bits, launcher verbs, ci trigger, version sites and tools; builds nothing
./build_app               # frontend, engine, frozen service, Rust shell, bundle
./build_app --clean       # rebuild everything from nothing
```

Flags: `--no-install` (macOS: leave the app where it is), `--install` (Linux: install the built deb with apt), `--open`. Linux bundles default to deb; set `FFB_BUNDLES=deb,rpm,appimage` for more. Every step logs to `build_app.log`. The first release engine build takes tens of minutes on a small machine. Bundles end up in `src-tauri/target/release/bundle`.

`./formfabric desktop dev` runs the app from source with the splash; `./formfabric desktop build` builds the installer and forwards `--config FILE` to Tauri; `./formfabric install-desktop` installs PyInstaller and checks the Rust and Tauri toolchain. The shell picks a free service port per launch; the engine makes the window key and the shell reads it from the engine's output and hands it to the window.

## formcompiler and formportal

formcompiler builds the app on three operating systems when a `compile_v*` branch is pushed (cut from `main`), requiring `ci.yml` to pass on that exact commit, then signs and drafts the release. formportal lists the app from the catalogue entry and connects its MCP server. Both are GPL-3.0 and are met by contract only: launcher verbs (`install`, `install-desktop`, `desktop build`), the CI trigger, a root `VERSION`, a changelog heading per version, the registry entry and stdio MCP. Never copy their code here. The drafts are `docs/integration/apps-formfabric.json` (registry), `catalogue-entry.json` and `product-entry.json`; their owners add them to their repositories, set `enabled` true with the first `compile_v` branch, and keep `id`, `name`, `kind`, `summary` and `identifier` identical across the registry, the catalogue and `tauri.conf.json`.

## What is verified and what is not

`./build_app --dry-run` passes on Linux. A full `build_app`, the three-operating-system builds, signing and notarization, and the formportal install and connect are owner runs. On Windows the contract path through `windows_shell cmd` is unproven, and `formfabric.bat py` and `mcp` are bash only. The `desktop dev` splash exit needs a display.

## Contract

Keep one version across all sites (`python3 setup/version.py check` verifies ten and the changelog heading). Keep `build_app` and the `setup/` scripts executable (the file mode is part of the contract). A new bundled resource must be added to `setup/freeze_backend` in both the bash and `.bat` forms.
