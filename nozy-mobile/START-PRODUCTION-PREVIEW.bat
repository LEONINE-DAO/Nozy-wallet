@echo off
title NozyWallet Mobile — production UI preview
cd /d C:\Users\User\NozyWallet\nozy-mobile
set PATH=C:\Program Files\nodejs;%PATH%
set EXPO_PUBLIC_APP_VARIANT=production

echo.
echo Production store UI preview (hosted API defaults, no experimental screens).
echo This is NOT the Play/App Store binary — use EAS for that:
echo   eas build --platform android --profile production
echo.
echo Optional: set NOZY_API_KEY in hosted-api.env before starting
echo   powershell -File ..\scripts\prepare-mobile-store-credentials.ps1
echo.

call npx expo start
pause
