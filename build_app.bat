@echo off
setlocal EnableExtensions
cd /d "%~dp0"
set "OPEN=0"
for %%A in (%*) do (
  if /i "%%~A"=="--open" ( set "OPEN=1" ) else if /i "%%~A"=="--help" ( goto :usage ) else if /i "%%~A"=="-h" ( goto :usage ) else ( call :usage & exit /b 2 )
)

echo.
echo [formfabric] == checking the tools
where python >nul 2>nul || ( echo [formfabric] Python 3.12 or newer is needed: https://www.python.org/downloads/ & exit /b 1 )
where node >nul 2>nul || ( echo [formfabric] Node 22 or newer is needed: https://nodejs.org & exit /b 1 )
where cargo >nul 2>nul || ( echo [formfabric] Rust is needed: https://rustup.rs & exit /b 1 )
for /f "tokens=1 delims=." %%V in ('node -p "process.versions.node"') do if %%V LSS 22 ( echo [formfabric] Node 22 or newer is needed, this is Node %%V: https://nodejs.org & exit /b 1 )
where claude >nul 2>nul && ( echo [formfabric] Claude Code found; the app's terminal dock and Chat can start it ) || ( echo [formfabric] Claude Code is not installed; the app builds without it: https://claude.com/claude-code )

echo.
echo [formfabric] == installing the service and the window
call backend\oc_install.bat || goto :failed
pushd frontend
if exist package-lock.json ( call npm ci --no-audit --no-fund || ( popd & goto :failed ) ) else ( call npm install --no-audit --no-fund || ( popd & goto :failed ) )
popd

echo.
echo [formfabric] == installing the desktop packages
call setup\install_desktop.bat || goto :failed

echo.
echo [formfabric] == building the Rust engine and its Python module
call setup\engine_build.bat || goto :failed

echo.
echo [formfabric] == building the window
call setup\build_frontend.bat || goto :failed

echo.
echo [formfabric] == freezing the service and engine into the app's sidecar
call setup\freeze_backend.bat || goto :failed

echo.
echo [formfabric] == building the desktop app (the first build compiles several hundred Rust crates)
if not exist src-tauri\icons\icon.ico call frontend\node_modules\.bin\tauri.cmd icon assets\formfabric_icon.png
call frontend\node_modules\.bin\tauri.cmd build || goto :failed

echo.
echo [formfabric] done. The installers are in src-tauri\target\release\bundle (msi and nsis)
if "%OPEN%"=="1" start "" "src-tauri\target\release\bundle"
exit /b 0

:failed
echo.
echo [formfabric] the build stopped at the step above; fix what it says and run build_app.bat again
exit /b 1

:usage
echo build_app.bat           build the formfabric desktop app on Windows, start to finish
echo   --open                open the folder with the installers when the build is done
exit /b 0
