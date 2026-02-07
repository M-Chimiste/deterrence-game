@echo off
setlocal

REM Deterrence — Dev Build (Windows wrapper)
REM Runs the PowerShell script even when .ps1 execution is restricted.

powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0dev.ps1" %*
exit /b %errorlevel%

