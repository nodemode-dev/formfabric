@echo off
setlocal
cd /d "%~dp0..\frontend"
if not exist node_modules call npm install
if not exist public\engine\ffb_wasm.wasm (
  where cargo >nul 2>nul && (
    pushd ..\native
    cargo build -p ffb-wasm --lib --release --target wasm32-unknown-unknown && (
      if not exist ..\frontend\public\engine mkdir ..\frontend\public\engine
      copy /y target\wasm32-unknown-unknown\release\ffb_wasm.wasm ..\frontend\public\engine\ffb_wasm.wasm >nul
    ) || echo [formfabric] the browser engine was not built; the window will need the engine service
    popd
  )
)
call npm run build || exit /b 1
echo [formfabric] built frontend\dist
