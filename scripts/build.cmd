@echo off
setlocal

REM Deterrence — Production Build (Windows wrapper)
REM Runs the PowerShell script even when .ps1 execution is restricted.

powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0build.ps1" %*
exit /b %errorlevel%

