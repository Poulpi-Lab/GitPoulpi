@echo off
rem Shortcut for scripts\build-custom.ps1: the interactive custom build (pick
rem languages, integrations, themes and features for a smaller binary).
rem Run `build-custom` from the repo root (PowerShell: `.\build-custom`).
pwsh -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\build-custom.ps1" %* || exit /b %ERRORLEVEL%
