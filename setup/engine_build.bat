@echo off
setlocal
cd /d "%~dp0..\native"
where cargo >nul 2>nul || ( echo [formfabric] Rust is not installed, get it from https://rustup.rs & exit /b 1 )
cargo build --workspace --release --exclude ffb-py --exclude ffb-wasm || exit /b 1
if not exist ..\backend\venv\Scripts\activate.bat ( echo [formfabric] backend venv missing, the python engine module was not built & exit /b 0 )
call ..\backend\venv\Scripts\activate.bat
where maturin >nul 2>nul || pip install maturin || exit /b 1
pushd crates\ffb-py
set "CARGO_TARGET_DIR=%~dp0..\native\target"
maturin develop --release || ( set "CARGO_TARGET_DIR=" & popd & exit /b 1 )
set "CARGO_TARGET_DIR="
popd
python -c "import ffb_engine; print('[formfabric] python engine module', ffb_engine.version())"
