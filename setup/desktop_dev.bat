@echo off
setlocal
cd /d "%~dp0.."
for /f "tokens=2" %%T in ('rustc -vV ^| findstr /b "host:"') do set "TRIPLE=%%T"
if not exist "src-tauri\binaries\formfabric-engine-%TRIPLE%.exe" call setup\freeze_backend.bat || exit /b 1
call frontend\node_modules\.bin\tauri.cmd dev %*
