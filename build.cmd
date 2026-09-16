@echo off
rem Shortcut for scripts\build.ps1: the daily incremental build (release
rem profile) of the binaries, with their sizes. Packaging: build-opti.
rem Run `build` from the repo root (PowerShell: `.\build`).
pwsh -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\build.ps1" %* || exit /b %ERRORLEVEL%
