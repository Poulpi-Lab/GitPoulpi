@echo off
rem Shortcut for scripts\build-opti.ps1: the official optimized build (dist
rem profile, thin LTO) of the three binaries, with their sizes.
rem Run `build-opti` from the repo root (PowerShell: `.\build-opti`).
pwsh -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\build-opti.ps1" %* || exit /b %ERRORLEVEL%
