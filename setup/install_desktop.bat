@echo off
setlocal
cd /d "%~dp0..\backend"
if not exist venv\Scripts\activate.bat ( echo [formfabric] backend venv missing, run formfabric.bat install & exit /b 1 )
call venv\Scripts\activate.bat
pip install -r requirements_desktop.txt || exit /b 1
where cargo >nul 2>nul || ( echo [formfabric] Rust is not installed, get it from https://rustup.rs & exit /b 1 )
if not exist ..\frontend\node_modules\.bin\tauri.cmd ( pushd ..\frontend & call npm install & popd )
if not exist ..\frontend\node_modules\.bin\tauri.cmd ( echo [formfabric] the Tauri CLI did not install & exit /b 1 )
echo [formfabric] desktop toolchain ready
