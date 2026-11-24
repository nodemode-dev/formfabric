# setup hashout log

- Windows engine without a console (0.27.0): `freeze_backend` and `freeze_backend.bat` give PyInstaller `--noconsole` on Windows unless `FFB_ENGINE_WINDOWED=0`. The engine's entry file replaces missing standard streams first, so the windowed build and `--selfcheck` still run.
- Dev boot (0.27.0): `boot_all` and `boot_all.bat` no longer make a window token; the service makes its own key and the window claims it.
- Keychain (0.28.0): `freeze_backend` and `freeze_backend.bat` add `--collect-submodules keyring`, because keyring finds its keychain backends by name at run time and a frozen engine without them would report the seal as unavailable.
- Test runner (0.31.0): `test` and `test.bat` run the test layers behind `./formfabric test [layer]`: `unit` is vitest in `frontend`, `engine` is pytest on `backend/tests` with the backend venv. No layer or `all` runs both, an unknown layer exits 2, and the rest of the line goes to the runner. The runner installs nothing: when `node_modules`, the venv, or pytest and httpx are missing it names the command and stops. The native workspace has no layer yet.
