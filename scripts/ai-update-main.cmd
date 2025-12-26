@echo off
setlocal
set SCRIPT_DIR=%~dp0
pwsh -NoProfile -ExecutionPolicy Bypass -File "%SCRIPT_DIR%ai-update-main.ps1" %*
endlocal
exit /b %errorlevel%


