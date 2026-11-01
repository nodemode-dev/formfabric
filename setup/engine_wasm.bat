@echo off
setlocal
set "ROOT=%~dp0.."
set "TARGET=wasm32-unknown-unknown"
where cargo >nul 2>nul || ( echo [formfabric] Rust is not installed, get it from https://rustup.rs & exit /b 1 )
for /f "delims=" %%S in ('rustc --print sysroot') do set "SYSROOT=%%S"
pushd "%ROOT%\native"
if exist "%SYSROOT%\lib\rustlib\%TARGET%\lib" (
  cargo build -p ffb-wasm --lib --release --target %TARGET% || ( popd & exit /b 1 )
) else if exist "%SYSROOT%\lib\rustlib\src\rust\library\std" (
  echo [formfabric] no prebuilt %TARGET% standard library, building it from source
  set "RUSTC_BOOTSTRAP=1"
  cargo build -p ffb-wasm --lib --release --target %TARGET% -Zbuild-std=std,panic_abort || ( popd & exit /b 1 )
) else (
  echo [formfabric] the %TARGET% target is missing, run rustup target add %TARGET% & popd & exit /b 1
)
popd
if not exist "%ROOT%\frontend\public\engine" mkdir "%ROOT%\frontend\public\engine"
copy /y "%ROOT%\native\target\%TARGET%\release\ffb_wasm.wasm" "%ROOT%\frontend\public\engine\ffb_wasm.wasm" >nul || exit /b 1
echo [formfabric] browser engine built at frontend\public\engine\ffb_wasm.wasm
