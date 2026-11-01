#!/usr/bin/env bash
FFB_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export FFB_ROOT
ffb_say() { printf '[formfabric] %s\n' "$*"; }
ffb_die() { printf '[formfabric] error: %s\n' "$*" >&2; exit 1; }
ffb_os() {
  case "$(uname -s 2>/dev/null)" in
    Darwin) echo macos ;;
    Linux) echo linux ;;
    MINGW*|MSYS*|CYGWIN*|*_NT-*) echo windows ;;
    *) echo unknown ;;
  esac
}
ffb_python() {
  local c
  for c in python3.14 python3.13 python3.12 python3 python; do
    if command -v "$c" >/dev/null 2>&1 && "$c" -c 'import sys; raise SystemExit(0 if sys.version_info >= (3, 12) else 1)' 2>/dev/null; then
      echo "$c"
      return 0
    fi
  done
  return 1
}
ffb_venv() {
  local act="$FFB_ROOT/backend/venv/bin/activate"
  [ -f "$act" ] || act="$FFB_ROOT/backend/venv/Scripts/activate"
  [ -f "$act" ] || ffb_die "backend venv missing, run ./formfabric install"
  # shellcheck disable=SC1090
  source "$act"
}
ffb_venv_python() {
  if [ -x "$FFB_ROOT/backend/venv/bin/python" ]; then echo "$FFB_ROOT/backend/venv/bin/python"
  elif [ -f "$FFB_ROOT/backend/venv/Scripts/python.exe" ]; then echo "$FFB_ROOT/backend/venv/Scripts/python.exe"
  else return 1; fi
}
ffb_target_triple() {
  if [ -n "${FFB_TARGET:-}" ]; then echo "$FFB_TARGET"; return 0; fi
  command -v rustc >/dev/null 2>&1 || return 1
  rustc -vV | awk '/^host:/ {print $2}'
}
ffb_exe() { [ "$(ffb_os)" = windows ] && echo ".exe" || echo ""; }
ffb_native_path() { if [ "$(ffb_os)" = windows ] && command -v cygpath >/dev/null 2>&1; then cygpath -w "$1"; else printf '%s' "$1"; fi; }
ffb_rust_env() {
  [ -d "$HOME/.cargo/bin" ] && PATH="$HOME/.cargo/bin:$PATH"
  export PATH
  unset RUSTC RUSTC_WORKSPACE_WRAPPER RUSTFLAGS CARGO_ENCODED_RUSTFLAGS CARGO_BUILD_RUSTFLAGS CARGO_BUILD_TARGET CARGO_TARGET_DIR CARGO_BUILD_TARGET_DIR CARGO_BUILD_RUSTC
  export RUSTC_WRAPPER="" CARGO_BUILD_RUSTC_WRAPPER=""
}
ffb_rust_fresh() {
  local target="$FFB_ROOT/src-tauri/target" now
  now="$(rustc -vV 2>/dev/null | tr '\n' ' ')"
  if [ -d "$target/release" ] && [ "$(cat "$target/.formfabric-rustc" 2>/dev/null)" != "$now" ]; then
    ffb_say "the Rust compiler differs from the one that built src-tauri/target; cleaning it"
    ( cd "$FFB_ROOT/src-tauri" && cargo clean )
  fi
  mkdir -p "$target"
  printf '%s' "$now" > "$target/.formfabric-rustc"
}
