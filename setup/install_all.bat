@echo off
setlocal
cd /d "%~dp0.."
echo [formfabric] installing backend
pushd backend
call oc_install.bat || ( popd & exit /b 1 )
popd
where cargo >nul 2>nul && ( call setup\engine_build.bat || exit /b 1 ) || echo [formfabric] Rust is not installed, the engine needs it: https://rustup.rs
echo [formfabric] installing frontend
pushd frontend
call oc_install.bat || ( popd & exit /b 1 )
popd
echo [formfabric] done. Boot with formfabric.bat
