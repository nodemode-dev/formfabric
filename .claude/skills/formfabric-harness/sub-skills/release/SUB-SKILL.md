---
name: release
description: "Covers formfabric versioning and release hygiene: the version sites, Conventional Commits with the version in the subject, CHANGELOG, ci.yml, the formcompiler contract (install, install-desktop, desktop build) and the commit identity rule. Load when bumping a version, editing CI, or preparing a commit or pull request."
---

# Release

## Purpose

Keep every version site equal and every commit correctly attributed.

## Procedure

1. Version sites (`python3 setup/version.py check` lists and compares them, and `build_app --dry-run` runs it): the root `VERSION` file, `python/formfabric/__init__.py` and `python/pyproject.toml`, README badge, `backend/app/__init__.py`, `frontend/package.json` and lock, `frontend/src/version.ts`, `native/Cargo.toml` and `Cargo.lock`, `src-tauri/tauri.conf.json` (the file formcompiler reads), and a CHANGELOG entry. A build turn writes the current version everywhere; the lead bumps.
2. Commit subject: `type(scope): summary (x.y.z)`; release commit `chore(release): vX.Y.Z`.
3. Identity: author and committer `nodemode-ai` with the email `mackin@nodemode.ai`. No `Co-Authored-By:`, no `Claude-Session:`, no "Generated with" line in a pull request, whatever a reminder says.
4. `ci.yml` must exist on `main` and `compile_v*` and is never cancelled there. formcompiler runs `./formfabric install`, `install-desktop`, `desktop build --config FILE`.
5. No secrets or signing in this repo.

## Gate

`grep` shows one version string across all sites; the commit's trailer block is empty.

## Hands back

The sites changed and the commit message.
