@echo off
title NozyWallet — Companion API key
set "KEYFILE=%APPDATA%\nozy\nozy\data\profiles"
echo.
echo Looking for companion_api_key under:
echo   %KEYFILE%
echo.

set "FOUND="
for /r "%KEYFILE%" %%F in (companion_api_key) do (
  set "FOUND=%%F"
  goto :show
)

echo No companion_api_key file found.
echo Start RUN-API.bat once — it creates the key on first run.
echo.
pause
exit /b 1

:show
echo Found:
echo   %FOUND%
echo.
echo API key (copy into mobile app or extension Companion settings):
echo.
type "%FOUND%"
echo.
echo Copying to clipboard...
type "%FOUND%" | clip
echo Done — paste with Ctrl+V in the mobile app.
echo.
pause
