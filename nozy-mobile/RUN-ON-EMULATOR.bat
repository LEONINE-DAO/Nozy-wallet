@echo off
title NozyWallet Mobile (Android emulator)
cd /d C:\Users\User\NozyWallet\nozy-mobile
set PATH=C:\Program Files\nodejs;C:\Program Files\Git\bin;%PATH%
set ANDROID_HOME=C:\Android\Sdk
set "ANDROID_SDK_ROOT="

echo.
echo NozyWallet needs Metro (the JS dev server) running before the wallet UI loads.
echo A debug APK shows the Expo "development build" screen until Metro is connected.
echo.

where adb >nul 2>&1
if %ERRORLEVEL%==0 (
  echo Waiting until the emulator is online ^(avoids device-offline spam^)...
  adb wait-for-device
  :waitBoot
  adb shell getprop sys.boot_completed 2>nul | findstr /r /c:"^1" >nul
  if errorlevel 1 (
    timeout /t 2 /nobreak >nul
    goto waitBoot
  )
  echo Forwarding emulator port 8081 to your PC...
  adb reverse tcp:8081 tcp:8081 >nul 2>&1
  adb reverse tcp:8082 tcp:8082 >nul 2>&1
  adb reverse tcp:7349 tcp:7349 >nul 2>&1
) else (
  echo WARNING: adb not in PATH — add Android SDK platform-tools if the app stays on the dev screen.
)

echo.
echo 1. Prefer START-EMULATOR.bat ^(uses -delay-adb^) instead of Device Manager Play
echo 2. Wait until that window says the emulator is online
echo 3. Metro will start below; the app should open on the emulator
echo    If you still see the dev screen, tap your project or enter: http://10.0.2.2:8081
echo.

call npx expo start --android
pause
