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
  echo Forwarding emulator port 8081 to your PC...
  adb reverse tcp:8081 tcp:8081 >nul 2>&1
  adb reverse tcp:8082 tcp:8082 >nul 2>&1
) else (
  echo WARNING: adb not in PATH — add Android SDK platform-tools if the app stays on the dev screen.
)

echo.
echo 1. Start NozyPixel in Android Studio ^(Device Manager - Play^)
echo 2. Wait for the Android home screen
echo 3. Metro will start below; the app should open on the emulator
echo    If you still see the dev screen, tap your project or enter: http://10.0.2.2:8081
echo.

call npx expo start --android
pause
