@echo off
setlocal

set "SCRIPT_DIR=%~dp0"
cd /d "%SCRIPT_DIR%\..\.."

where hyper-top >nul 2>nul
if not errorlevel 1 (
  hyper-top %*
  exit /b %errorlevel%
)

cargo run -- %*
exit /b %errorlevel%
