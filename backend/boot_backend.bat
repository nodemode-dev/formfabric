@echo off
cd /d "%~dp0"
if not exist venv\Scripts\activate.bat (echo [formfabric] backend venv missing, run formfabric.bat install & exit /b 1)
call venv\Scripts\activate.bat
if "%FFB_PORT%"=="" set "FFB_PORT=8280"
uvicorn app.main:app --reload --timeout-graceful-shutdown 1 --host 127.0.0.1 --port %FFB_PORT%
