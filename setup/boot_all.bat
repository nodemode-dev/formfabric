@echo off
setlocal
cd /d "%~dp0.."
start "formfabric backend" cmd /c "cd backend && boot_backend.bat"
cd frontend
call boot_frontend.bat
