@echo off
setlocal
cd /d "%~dp0"
set "CMD=%~1"
if "%CMD%"=="" set "CMD=boot"
shift
if /i "%CMD%"=="boot" ( call setup\boot_all.bat & goto :done )
if /i "%CMD%"=="dev" ( call setup\boot_all.bat & goto :done )
if /i "%CMD%"=="install" ( call setup\install_all.bat & goto :done )
if /i "%CMD%"=="install-desktop" ( call setup\install_desktop.bat & goto :done )
if /i "%CMD%"=="backend" ( call setup\boot_backend.bat & goto :done )
if /i "%CMD%"=="frontend" ( call setup\boot_frontend.bat & goto :done )
if /i "%CMD%"=="build" ( call setup\build_frontend.bat & goto :done )
if /i "%CMD%"=="freeze" ( call setup\freeze_backend.bat & goto :done )
if /i "%CMD%"=="app" ( call build_app.bat %1 %2 %3 & goto :done )
if /i "%CMD%"=="desktop" goto :desktop
if /i "%CMD%"=="engine" goto :engine
if /i "%CMD%"=="help" ( call :usage & set "RC=0" & goto :done )
if /i "%CMD%"=="serve" goto :bashonly
if /i "%CMD%"=="mcp" goto :bashonly
if /i "%CMD%"=="py" goto :bashonly
if /i "%CMD%"=="agents" goto :bashonly
if /i "%CMD%"=="i18n" goto :bashonly
if /i "%CMD%"=="doctor" goto :bashonly
if /i "%CMD%"=="clean" goto :bashonly
echo [formfabric] unknown command: %CMD%
call :usage
set "RC=2"
goto :done

:desktop
set "SUB=%~1"
if "%SUB%"=="" set "SUB=dev"
shift
set "REST="
:collect
if "%~1"=="" goto :collected
set REST=%REST% "%~1"
shift
goto :collect
:collected
if /i "%SUB%"=="build" ( call setup\desktop_build.bat %REST% & goto :done )
if /i "%SUB%"=="dev" ( call setup\desktop_dev.bat %REST% & goto :done )
echo [formfabric] unknown desktop command: %SUB%
set "RC=2"
goto :done

:engine
set "SUB=%~1"
if "%SUB%"=="" set "SUB=build"
if /i "%SUB%"=="build" ( call setup\engine_build.bat & goto :done )
goto :bashonly

:bashonly
echo [formfabric] "%CMD%" is a bash command. Open Git Bash in this folder and run: ./formfabric %CMD% %1 %2 %3
set "RC=2"
goto :done

:usage
echo formfabric.bat [command]
echo   boot ^| install ^| install-desktop ^| backend ^| frontend ^| build ^| freeze
echo   engine build ^| desktop dev ^| desktop build [--config FILE] ^| app ^| help
echo   serve, mcp, agents, i18n, doctor, clean and engine check^|wasm run from Git Bash: ./formfabric
exit /b 0

:done
if not defined RC set "RC=%ERRORLEVEL%"
endlocal & exit /b %RC%
