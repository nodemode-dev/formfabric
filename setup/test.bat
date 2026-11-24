@echo off
setlocal
set "ROOT=%~dp0.."
set "WHICH=%~1"
if "%WHICH%"=="" set "WHICH=all"
set "REST=%2 %3 %4 %5 %6 %7 %8 %9"
set "FAIL=0"
set "KNOWN=0"
for %%L in (all unit engine) do if /i "%WHICH%"=="%%L" set "KNOWN=1"
if "%KNOWN%"=="0" ( echo [formfabric] unknown test layer: %WHICH% ^(use unit, engine or nothing for all^) & exit /b 2 )
set "DO=0"
if /i "%WHICH%"=="all" set "DO=1"
if /i "%WHICH%"=="unit" set "DO=1"
if "%DO%"=="1" (
  if not exist "%ROOT%\frontend\node_modules\vitest" ( echo [formfabric] error: frontend not installed, run formfabric.bat install & exit /b 1 )
  echo [formfabric] interface logic ^(vitest^)
  pushd "%ROOT%\frontend"
  call npx vitest run %REST% || set "FAIL=1"
  popd
)
set "DO=0"
if /i "%WHICH%"=="all" set "DO=1"
if /i "%WHICH%"=="engine" set "DO=1"
set "PY=%ROOT%\backend\venv\Scripts\python.exe"
if "%DO%"=="1" (
  if not exist "%PY%" ( echo [formfabric] error: backend venv missing, run formfabric.bat install & exit /b 1 )
  "%PY%" -c "import pytest, httpx" >nul 2>&1 || ( echo [formfabric] error: pytest and httpx missing from the backend venv, run: "%PY%" -m pip install pytest httpx & exit /b 1 )
  echo [formfabric] service ^(pytest^)
  pushd "%ROOT%\backend"
  "%PY%" -m pytest tests -q -p no:cacheprovider %REST% || set "FAIL=1"
  popd
)
exit /b %FAIL%
