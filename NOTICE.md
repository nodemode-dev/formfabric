# Third-party notices

formfabric is licensed under the Apache License 2.0 (see `LICENSE`). Copyright 2026 NodeAgentc, Inc.

## Included in this repository

| Work | License | Where |
| --- | --- | --- |
| formvis desktop shell (nodemode-ai) | Apache-2.0 | `src-tauri/src/{reveal,traffic_lights,text_edit,window_state,packaged,main}.rs`, `src-tauri/build.rs`, `capabilities/`, `entitlements.plist`, `about.*`, `frontend/public/splash.html` (geometry and timing), `setup/freeze_backend` (adapted) |
| formvis window shell (nodemode-ai) | Apache-2.0 | `frontend/index.html` (theme boot), `frontend/src/ui/{dom,dialogs,dialog-history,dropdown,number-fields,toast,menu,native-menu,caption-buttons,commands,command-palette,i18n,themes,icons}.ts`, `frontend/src/ui/i18n/` (core tables), `frontend/src/styles/` (tokens and shell rules) (adapted) |
| formsculpt scripts, service, dock and engine pieces (nodemode-ai) | Apache-2.0 | `formfabric`, `formfabric.bat`, `build_app`, `setup/`, `backend/app/` (core, terminal, Claude Code, chat, agents, MCP), `frontend/src/dock/`, `frontend/src/styles/dock.css`, `native/crates/ffb-core/src/{bvh,mesh}.rs`, the registry and history shape in `native/crates/ffb-commands`, `ffb-py`, `ffb-wasm`, `frontend/src/workspace/view3d/{camera,math}.ts`, `.claude/` and `.agents/` kit layout, `setup/agents.py` (adapted) |
| NodeMode design system tokens | MIT | `frontend/src/styles/tokens.css` (values) |

## Referenced, not included

`docs/encyclopedia/` describes Marvelous Designer (CLO Virtual Fashion Inc.) from public sources, paraphrased and linked. No code, artwork, avatars, presets or manual text from it are included. Marvelous Designer and CLO are trademarks of CLO Virtual Fashion Inc.

## Distributed with the app

| Package | License | Notes |
| --- | --- | --- |
| Inter, JetBrains Mono (fontsource 5.3.0) | OFL-1.1 | bundled fonts |
| @xterm/xterm 6.0.0, @xterm/addon-fit 0.11.0 | MIT | terminal dock |
| @tauri-apps/api, @tauri-apps/plugin-dialog | MIT or Apache-2.0 | window bridge, file dialogs |
| Tauri 2 crates and plugins (updater, dialog, shell, single instance) | MIT or Apache-2.0 | desktop shell |
| glam, serde, serde_json, regex | MIT or Apache-2.0 | engine and shell |
| zip, flate2, miniz_oxide, zopfli, adler2 | MIT, Zlib, Apache-2.0, 0BSD | `.ffbrc` container |
| tungstenite, getrandom, rand, sha1 and the RustCrypto support crates | MIT or Apache-2.0 | frame stream |
| pyo3 | MIT or Apache-2.0 | Python engine module |
| FastAPI, Starlette, pydantic | MIT, BSD-3-Clause | engine service |
| uvicorn, websockets, python-dotenv, click | BSD-3-Clause | engine service |
| python-multipart | Apache-2.0 | engine service |
| ptyprocess, pywinpty | ISC, MIT | terminal |

Build tools that are not distributed: Vite (MIT), TypeScript (Apache-2.0), maturin (MIT or Apache-2.0), PyInstaller (GPL-2.0 with the bootloader exception).

This file grows with each phase: every adapted family file and every distributed dependency is listed when it is added.
