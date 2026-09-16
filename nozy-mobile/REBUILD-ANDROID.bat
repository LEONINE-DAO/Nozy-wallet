@echo off
title NozyWallet — rebuild Android native app
cd /d C:\Users\User\NozyWallet\nozy-mobile
set PATH=C:\Program Files\nodejs;C:\Program Files\Git\bin;%USERPROFILE%\.cargo\bin;%PATH%
set ANDROID_HOME=C:\Android\Sdk
set ANDROID_SDK_ROOT=C:\Android\Sdk
if exist "C:\Users\User\AppData\Local\Android\Sdk" (
  set ANDROID_HOME=C:\Users\User\AppData\Local\Android\Sdk
  set ANDROID_SDK_ROOT=C:\Users\User\AppData\Local\Android\Sdk
)

echo.
echo This compiles Kotlin + libnozy_ffi.so and installs on the emulator/device.
echo Expo Go and RUN-ON-EMULATOR.bat cannot load the on-device wallet.
echo Start the Android emulator first (Android Studio Device Manager - Play).
echo.

where adb >nul 2>&1
if %ERRORLEVEL%==0 (
  adb reverse tcp:8081 tcp:8081 >nul 2>&1
  adb reverse tcp:8082 tcp:8082 >nul 2>&1
)

call npx expo run:android
if errorlevel 1 (
  echo.
  echo Rebuild failed. Common fixes:
  echo   - Emulator running with the home screen showing
  echo   - Android Studio SDK at ANDROID_HOME
  echo   - From this folder: npm install
  echo.
)
pause
