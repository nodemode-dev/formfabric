# Contributing to FormFabric

Thank you for helping. FormFabric is the Form family's garment app: draft sewing
patterns, sew them edge to edge, drape them on a body with a cloth simulation, animate
and export, in one window with no account, running on your own computer, and open to
agents over MCP. It gets better through real use and real contributions.

## Where to start

- **Use it and report what goes wrong.** A clear bug report, with steps, the version
  and your platform, is one of the most useful things you can give.
- **Look at the open issues.** Comment on one before starting larger work, so nobody
  builds the same thing twice.
- **Translations** are welcome: `./formfabric i18n check` checks each language table
  against `strings.json`, and a new interface string lands in every table under
  `frontend/src/ui/i18n/` in the same change.

## Setting up

Clone the repository, run `./formfabric_oc_install` once, then `./formfabric` (on
Windows, `formfabric_oc_install.bat` and `formfabric.bat`). The first installs the
backend, the engine and the frontend; the second starts the service on 127.0.0.1:8280
and the window at http://localhost:5280. You need Node 20.19 or newer (22
recommended), Python 3.12 or newer, and Rust at the version pinned by
`rust-toolchain.toml`; `./formfabric doctor` checks them. `./formfabric desktop dev`
opens the desktop window. The README's "Run it from source" section has the details.

The code lives in:

| Folder | What is there |
| --- | --- |
| `native/crates` | the engine (Rust): the garment model, the cloth solver, bodies, animation, remesh, file formats and the command registry `ffb-commands` |
| `frontend/src` | the window (TypeScript, Vite): `shell/`, `ui/`, `dock/`, `engine/` and `workspace/` |
| `backend/app` | the service (Python, FastAPI): engine dispatch, the garment library, autosave and the recovery journal, plugins, MCP, the docks |
| `backend/plugins` | the plugins shipped with the app |
| `python` | the standard-library `formfabric` Python package |
| `src-tauri` | the desktop shell (Rust, Tauri 2) |
| `setup` | the scripts behind `./formfabric` |
| `assets` | the app icon, the CC0 bodies and the sample garments |
| `docs/integration` | drafts for other repositories (registry, catalogue and product entries, hand-offs) |
| `.claude/skills` | agent skills, mirrored into `.agents/skills` |
| `.docs/wiki` | the wiki's source, published on push |

## Before you open a pull request

Run the same checks CI runs, and the checks `CLAUDE.md` names:

```sh
(cd frontend && npm ci && npm run build)
./formfabric engine check
./formfabric engine wasm
./formfabric engine build
(cd backend && venv/bin/python formfabric_engine.py --selfcheck)
./formfabric i18n check
./formfabric agents sync --check
python3 setup/version.py check
./build_app --dry-run
```

CI also runs `cargo check --locked` over the desktop shell on Linux. A release engine
build is long; use a private `CARGO_TARGET_DIR` when several builds share a machine.

- Keep each pull request to one change, and update its wiki page under `.docs/wiki/`
  in the same pull request when a command, setting or screen changes.
- Commit subjects follow the history: `type(scope): plain sentence (X.Y.Z)`, for
  example `feat(window): ...`, `fix(engine): ...`, `docs(hashout): ...`. Version
  bumps, `CHANGELOG.md` and the release belong to the maintainers; leave the version
  sites alone. See [.docs/BRANCHING.md](.docs/BRANCHING.md).
- A command is declared only in `native/crates/ffb-commands`. After adding or changing
  one, run `./formfabric engine build`, then `./formfabric agents sync`, and refresh
  `python/formfabric/catalogue.json` with `./formfabric py -m formfabric snapshot`.
- A change in behaviour updates its tests in the same pull request; `./formfabric test`
  runs them, and [docs/TESTING.md](docs/TESTING.md) says what a test must do to be accepted.
- Icons are inline `currentColor` SVG in `frontend/src/ui/icons.ts`, every `z-index`
  is a `--z-*` token from `frontend/src/styles/tokens.css`, and Vite binds localhost
  only.
- No emoji anywhere, no em dash in Markdown, and keep code comments to a minimum: the
  context goes in the side's `HASHOUT_LOG.md` (`frontend/`, `backend/`, `native/`).
  `README.md` uses the house compression style; every other Markdown file is written
  in full words.
- The three-star sparkle icon is not used anywhere in the app; never add one.
- formcompiler and formportal are GPL-3.0. FormFabric meets them through contracts
  only (launcher verbs, `ci.yml`, registry and catalogue entries kept in their
  repositories, stdio MCP). Never copy their code into this tree.

## How changes land

Only the maintainers push to `main`. Everyone else forks the repository and opens a
pull request into `main`. A pull request merges when every CI check passes on an
up-to-date branch and a maintainer has approved it after its last push.

## Releases

A release is a version, and only the maintainers make one: the version bump is the last
commit on `main`, a maintainer cuts a `compile_vX.Y.Z` branch from it, and every build
is compiled and released from
[FormCompiler](https://github.com/nodemode-dev/formcompiler), which builds every
platform and publishes them together. This repository holds the source; packaging and
releases stay in FormCompiler. See [.docs/BRANCHING.md](.docs/BRANCHING.md).

## Licensing

FormFabric is licensed under the Apache License 2.0 ([LICENSE](LICENSE), SPDX
`Apache-2.0`). By contributing, you agree that your contribution is licensed under the
same terms. Only add code you have the right to contribute, and name any third-party
code and its licence in the pull request.

## Patents and other people's IP

Some features are deliberately not built because they come too close to someone else's
patent, trademark, copyright or design. The maintainers hold that line with counsel;
contributors do not decide it, and nobody here redesigns a feature on their own reading
of a patent. This is engineering diligence, not legal advice.

### Rules for every contribution

- Patents: do not add a feature, or a dependency that provides one, that reproduces a method you know to be patented. If you are unsure, open an issue that describes what the feature does, not which patent it resembles. Patent numbers stay out of code, commits, comments, logs and changelogs.
- Trademarks: other vendors' product and feature names appear only to name a file format or compatibility ("opens <format> files", "for use with"). No command, menu, panel, tool, preset, template, theme or agent tool takes another product's feature name, and no text says "like X" or "the way X does it". Describe what the feature does. FormFabric's features carry its own names.
- Copyright: never copy another product's help, tooltip, dialog or documentation text (link to it instead); never trace or import vendor icons; never bundle vendor templates, themes, presets, avatars, fabrics, sample files, stock media or fonts, and never copy another product's file format. Sample garments and fixtures come from this repository's own work.
- Licences: a dependency must be MIT, Apache-2.0, BSD, ISC, Zlib, OFL, Unicode or MPL-2.0 (file-level). No GPL, LGPL or AGPL code is linked or copied. Every new dependency, binary, model, body, font or data file gets a row in `NOTICE.md` with its licence in the same change. Assets are CC0 or CC BY with the credit recorded; CC NC and ND never.

## Conduct

Everyone taking part follows the [code of conduct](CODE_OF_CONDUCT.md). Security
problems go through [SECURITY.md](SECURITY.md), never a public issue.
