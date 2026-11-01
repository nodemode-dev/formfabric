# formfabric

**Garment pattern design and cloth simulation, in the NodeMode style**

![version](https://img.shields.io/badge/version-0.2.0-f97316) ![status](https://img.shields.io/badge/status-in%20development-f97316) ![license](https://img.shields.io/badge/license-Apache--2.0-blue)

## What it is

formfabric is the form family's garment app: draw sewing patterns in 2D, sew them, drape them on bodies with a GPU cloth simulation, animate them and export them. It aims at full feature parity with Marvelous Designer, built in the open under Apache-2.0.

Planned stack · Rust engine (`native/`, one Cargo workspace, wgpu compute solver) · vanilla TypeScript and Vite (`frontend/`) · Python and FastAPI (`backend/`) · Tauri 2 (`src-tauri/`). Desktop builds for macOS, Windows and Linux come from formcompiler; formportal lists and connects the app.

## Status

v0.0.1 is the planning release. The repo holds the research encyclopedia and the license; the application is built in phases once the plan of record is approved.

## The encyclopedia

`docs/encyclopedia/` documents what Marvelous Designer does and how each feature behaves, from primary sources only, as the specification formfabric is built against.

| Start here | |
| --- | --- |
| [00-INDEX.md](docs/encyclopedia/00-INDEX.md) | chapter list and reading order |
| [14-PARITY-MATRIX.md](docs/encyclopedia/14-PARITY-MATRIX.md) | every feature mapped to a formfabric module, name and phase |
| [15-OPEN-GAPS.md](docs/encyclopedia/15-OPEN-GAPS.md) | what could not be verified and how formfabric decides it |

## Trademarks

Marvelous Designer and CLO are trademarks of CLO Virtual Fashion Inc., named here only to describe compatibility and the reference studied. formfabric is not affiliated with or endorsed by CLO Virtual Fashion.

## License

Apache-2.0, see `LICENSE`. Copyright 2026 NodeAgentc, Inc. Third-party notices in `NOTICE.md`.
