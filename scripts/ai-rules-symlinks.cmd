@echo off
setlocal
set SCRIPT_DIR=%~dp0
pwsh -NoProfile -ExecutionPolicy Bypass -File "%SCRIPT_DIR%ai-rules-symlinks.ps1" %*
endlocal
exit /b %errorlevel%


