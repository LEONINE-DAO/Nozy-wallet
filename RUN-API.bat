@echo off
title NozyWallet API
cd /d C:\Users\User\NozyWallet
echo.
echo Starting NozyWallet API on port 3000...
echo KEEP THIS WINDOW OPEN while using the app.
echo.
target\release\nozywallet-api.exe
pause
