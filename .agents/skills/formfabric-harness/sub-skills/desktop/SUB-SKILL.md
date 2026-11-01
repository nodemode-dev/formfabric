---
name: desktop
description: "Covers the formfabric Tauri 2 desktop shell in src-tauri/: identifier ai.nodemode.formfabric, splash and reveal handshake (formfabricSplashDone), sidecar spawn and port discovery, window chrome, .ffbrc file association, PyInstaller freezing of formfabric-engine. Load when editing src-tauri, splash.html, build_app or freeze scripts."
---

# Desktop

## Purpose

Own the thin Tauri shell over the frozen engine sidecar. Hands back a shell that boots, shows the splash once and reveals the main window without a flash.

## Procedure

1. Identifier `ai.nodemode.formfabric`; extension `.ffbrc`, MIME `application/x-formfabric`.
2. Sidecar `formfabric-engine`: one-folder on macOS, `src-tauri/binaries/formfabric-engine-[triple]` elsewhere. The shell reads the port from the uvicorn line.
3. Splash: 360 by 200 card, minimum 2600 ms, bar finish 220 ms, fallback 20 s. Keep `formfabricSplashDone` identical in `splash.html` and `reveal.rs`.
4. Colours: Studio Dark, accent #f97316; icon and splash gradient #f97316, #ffb347, #e8577a, #7c5cff.
5. No signing, notarization, updater keys or release workflows in this repo; formcompiler owns them.

## Gate

`cargo check` in src-tauri, the sidecar selfcheck exits 0, splash handshake names match.

## Hands back

The edit and the check output.
