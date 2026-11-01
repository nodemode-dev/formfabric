---
name: engine
description: "Covers the formfabric native engine in native/: ffb-core, ffb-commands registry, Session, history, catalogue and dispatch, ffb-io and the .ffbrc zip, ffb-py (ffb_engine) and ffb-wasm bindings. Load when adding or changing a command, the document model, undo, file IO or either binding."
---

# Engine

## Purpose

Own `native/`. One Cargo workspace (edition 2021, Apache-2.0, authors nodemode-ai). Units are millimetres, f64, y up. Hands back a change that builds natively and for wasm32-unknown-unknown without threads.

## Procedure

1. Read `native/Cargo.toml` and the crate you touch.
2. A command is a `Command { id, mcp, group, label, description, keys, hints, schema, run }` exported by a `*_cmds.rs` module and concatenated in `lib.rs`. Ids are dotted `group.verb_noun`, MCP names snake case. Groups: System, File, History, View, Pattern, Sew, Sim, Body, Fabric, Anim, Export.
3. `dispatch(name_or_id, args_json)` returns `{ok,result}` or `{ok:false,error:{code,message}}`; codes `unknown_command` and `bad_args`.
4. Every mutating command records exactly one history step; a drag groups into one.
5. `.ffbrc` is a zip with `manifest.json` and `garment.json`; unknown entries survive a save.
6. Never call `std::time::Instant` in code that reaches wasm; keep threads behind a feature.
7. After a registry change run `./formfabric agents sync`.

## Gate

Builds on the 1.97 and 1.99.1 toolchains, clippy clean, catalogue lists the command, undo then redo restores the document.

## Hands back

The edit, the catalogue diff, the check output.
