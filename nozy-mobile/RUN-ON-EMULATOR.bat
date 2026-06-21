@echo off
title NozyWallet Mobile
cd /d C:\Users\User\NozyWallet\nozy-mobile
set PATH=C:\Program Files\nodejs;C:\Program Files\Git\bin;%PATH%
echo.
echo 1. Start NozyPixel in Android Studio first (Device Manager - Play button)
echo 2. Wait for Android HOME screen
echo 3. This will open NozyWallet on the emulator...
echo.
call npx expo start --android
pause
