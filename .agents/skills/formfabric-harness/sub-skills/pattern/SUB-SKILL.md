---
name: pattern
description: "Covers formfabric 2D pattern work: the Garment, Panel, Point and Inner model, pattern commands (draw_panel, rectangle, ellipse, move_point, set_handle, transform, measure, snap and the rest), Bezier handles, and the Pattern View canvas with wasm drag previews. Load when editing ffb-pattern, pattern_cmds.rs or frontend/src/workspace/pattern-view."
---

# Pattern

## Purpose

Own the 2D pattern model and tools. Behaviour is specified in `docs/encyclopedia/02-pattern-2d.md` and `02b-pattern-2d-gaps.md`; names come from `14-PARITY-MATRIX.md`.

## Procedure

1. Model: `Garment { units, panels, selection }`; `Panel` has id, name, outline, inner, position, rotation, mirror, grain, particle_distance, layer; outline points are panel-local mm.
2. `Point` has `p`, `kind` (corner, curve, smooth) and optional `h_in` and `h_out`; a segment is a cubic Bezier when either end has a handle, else a line.
3. Commands live in `ffb-commands/src/pattern_cmds.rs` with schemas rich enough to drive an Exact Input dialog. Geometry lives in `ffb-pattern`, never in the command file.
4. Keys follow chapter 02: A transform, Z edit point, F frame.
5. Pattern View draws on one canvas from a retained model. A drag previews through wasm (`engine/wasm.ts`) and commits one service command on release.
6. Check `15-OPEN-GAPS.md` before choosing an unpublished default and record the choice.

## Gate

Round trip through `pattern.get` and `ffb_load` matches, one history step per gesture, no DOM node per shape.

## Hands back

The edit and the command names touched.
