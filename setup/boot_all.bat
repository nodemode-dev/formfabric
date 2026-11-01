@echo off
setlocal
cd /d "%~dp0.."
if "%FFB_WINDOW_TOKEN%"=="" for /f %%i in ('powershell -NoProfile -Command "([guid]::NewGuid().ToString('N') + [guid]::NewGuid().ToString('N'))"') do set "FFB_WINDOW_TOKEN=%%i"
start "formfabric backend" cmd /c "cd backend && boot_backend.bat"
cd frontend
call boot_frontend.bat
