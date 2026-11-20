# setup hashout log

- Windows engine without a console (0.27.0): `freeze_backend` and `freeze_backend.bat` give PyInstaller `--noconsole` on Windows unless `FFB_ENGINE_WINDOWED=0`. The engine's entry file replaces missing standard streams first, so the windowed build and `--selfcheck` still run.
- Dev boot (0.27.0): `boot_all` and `boot_all.bat` no longer make a window token; the service makes its own key and the window claims it.
