# Recovery

formfabric autosaves the project and keeps a crash journal, so a killed engine process loses at most one autosave interval plus nothing that was logged.

## Where it lives

| Path | Holds |
| --- | --- |
| `backend/app/services/autosave.py` | the autosave loop (`FFB_AUTOSAVE_SECONDS`, default 180) |
| `backend/app/services/recovery.py` | the journal and the replay |
| `backend/app/routers/recovery.py` | `GET /api/recovery`, `POST /api/recovery/restore`, `POST /api/recovery/discard` |
| `backend/app/services/engine.py` | records each mutating command and cuts the log at an autosave |
| `frontend/src/shell/recovery.ts` | the Restore or Discard dialog |

## What is kept

In `[data dir]/autosave/`: `autosave.ffbrc`, `journal.json` (clean flag, start time, autosave time) and `journal.jsonl` (one line per mutating command: number, time, id, arguments). After a crash, the old journal moves to `recovery/` with a copy of the autosave and `info.json`.

Logged: every successful command that changed the project, except the `sim` and `export` groups, `file_save`, library changes, `component_save`, `fabric_export`, animation recording and playback, and `link_` commands. String arguments over 8192 characters (captures, images) are dropped. The log is cut when an autosave completes, so it always holds the commands since that autosave.

## What you see

On the next start, after the catalogue loads, the window asks `GET /api/recovery`. If a crash was found a dialog above the launcher shows the autosave time and the number of logged commands with Restore and Discard. Restore opens the autosave (or starts empty when the crash came before the first autosave), replays the log (failures are listed, at most 20), writes a fresh autosave and clears the journal. Discard just clears it. All three routes need the window token. A clean shutdown marks the journal clean.

## Checked

The service was killed with signal 9 during `sim_start`; on restart it reported five logged commands and the restore replayed five of five, with the garment equal to the original apart from drape positions. Saved layouts survive restarts (they live in preferences, not the project).

## Limits

Drape positions are not journalled (they are not commands), so a restored garment is at its placed state. Replay depends on files that `file_open` and `library_insert` named still existing. A backend without the router answers 404, which shows once in the console.

## Contract

A new mutating command is journalled automatically; a new command that must not be replayed has to be added to `SKIP_IDS`, `SKIP_PREFIXES` or `SKIP_GROUPS` in `backend/app/services/recovery.py`.
