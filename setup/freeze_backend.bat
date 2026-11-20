@echo off
setlocal
cd /d "%~dp0.."
set "ROOT=%CD%"
if not exist backend\venv\Scripts\activate.bat ( echo [formfabric] backend venv missing, run formfabric.bat install & exit /b 1 )
call backend\venv\Scripts\activate.bat
python -c "import PyInstaller" >nul 2>nul || ( echo [formfabric] PyInstaller missing, run formfabric.bat install-desktop & exit /b 1 )
for /f "tokens=2" %%T in ('rustc -vV ^| findstr /b "host:"') do set "TRIPLE=%%T"
if defined FFB_TARGET set "TRIPLE=%FFB_TARGET%"
if not defined TRIPLE ( echo [formfabric] rustc not found, install Rust or set FFB_TARGET & exit /b 1 )
if not exist frontend\dist\index.html call setup\build_frontend.bat || exit /b 1
pushd backend
python -c "import ffb_engine, app; assert ffb_engine.version() == app.__version__" >nul 2>nul || ( popd & call setup\engine_build.bat || exit /b 1 & pushd backend )
if exist build\pyinstaller rmdir /s /q build\pyinstaller
if exist dist-engine rmdir /s /q dist-engine
set "EXTRA="
if exist "%ROOT%\setup\agents.py" set EXTRA=--add-data "%ROOT%\setup\agents.py;setup"
if exist "%ROOT%\.claude\skills" set EXTRA=%EXTRA% --add-data "%ROOT%\.claude\skills;.claude\skills"
if exist "%ROOT%\backend\plugins\*.py" set EXTRA=%EXTRA% --add-data "%ROOT%\backend\plugins;plugins"
if exist "%ROOT%\assets\samples" set EXTRA=%EXTRA% --add-data "%ROOT%\assets\samples;assets\samples"
if not "%FFB_ENGINE_WINDOWED%"=="0" set EXTRA=%EXTRA% --noconsole
pyinstaller --noconfirm --clean --onefile --name formfabric-engine --distpath dist-engine --workpath build\pyinstaller --specpath build\pyinstaller --add-data "%ROOT%\frontend\dist;frontend_dist" %EXTRA% --collect-submodules app --collect-submodules uvicorn --collect-all ffb_engine --collect-submodules keyring --hidden-import app.services.cmd_terminal --hidden-import app.services.claude_code --hidden-import app.services.cmd_chat --hidden-import winpty "%ROOT%\backend\formfabric_engine.py" || ( popd & exit /b 1 )
popd
if not exist src-tauri\binaries mkdir src-tauri\binaries
copy /y backend\dist-engine\formfabric-engine.exe "src-tauri\binaries\formfabric-engine-%TRIPLE%.exe" >nul || exit /b 1
"src-tauri\binaries\formfabric-engine-%TRIPLE%.exe" --selfcheck || exit /b 1
echo [formfabric] sidecar ready: src-tauri\binaries\formfabric-engine-%TRIPLE%.exe
