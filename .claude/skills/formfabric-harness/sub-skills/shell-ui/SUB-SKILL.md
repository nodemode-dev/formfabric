---
name: shell-ui
description: "Covers the formfabric frontend shell in frontend/: vanilla TypeScript and Vite, family tokens and themes, menus, command palette (Mod+K), shortcut editor (Mod+/), Inspector, status bar, workspace layout, dock, icons and i18n. Load when editing frontend/src, styles or frontend/public."
---

# Shell UI

## Purpose

Own `frontend/`. Vanilla TypeScript, no framework, no icon library. Hands back UI that never flashes or shifts.

## Procedure

1. `app.ts` stays small: it builds the shell, then calls `mountWorkspace(host, deps)` and `mountDock(host, deps)`.
2. `ui/commands.ts` builds the command list from the catalogue plus window-only commands; menus, palette, shortcuts and Exact Input read it.
3. `engine/service.ts` exports `call`, `catalogue` and `onEngineChanged`; the Vite proxy forwards `/api` to 8280 in the browser build.
4. Tokens in `styles/tokens.css`; every z-index is a `--z-*` token. Shared keys `nodemode_ui_settings` and `nm:ui-settings-changed`; app keys use the prefix `formfabric.`.
5. Icons are inline SVG with `currentColor` in one module. Never the three-star sparkle.
6. Apply the theme before first paint. Write one node and one property at a time; reserve space before content arrives.
7. Strings go through i18n; no other product's names in labels.

## Gate

Type check and build pass; a throttled reload shows no flash or layout jump; `server.host` is false.

## Hands back

The edit and what was checked.
