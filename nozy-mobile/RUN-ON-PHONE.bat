@echo off
title NozyWallet Mobile (USB phone)
cd /d C:\Users\User\NozyWallet\nozy-mobile
set PATH=C:\Program Files\nodejs;C:\Program Files\Git\bin;%PATH%
set ANDROID_HOME=C:\Users\User\AppData\Local\Android\Sdk
if exist "C:\Android\Sdk" set ANDROID_HOME=C:\Android\Sdk
set ANDROID_SDK_ROOT=%ANDROID_HOME%
set PATH=%ANDROID_HOME%\platform-tools;%PATH%

echo.
echo On-device Nozy on a real phone: keys stay on the phone.
echo Sync uses https://lwd.nozywallet.org:443 over the phone's Wi-Fi or mobile data.
echo Companion API is optional. Never use 10.0.2.2 on a physical phone.
echo.
echo 1. Enable Developer options, then USB debugging
echo 2. Plug in the phone, unlock it, tap Allow on the RSA prompt
echo 3. Keep Metro running on this PC (this script starts it if needed)
echo.

where adb >nul 2>&1
if not %ERRORLEVEL%==0 (
  echo ERROR: adb not found. Set ANDROID_HOME to the Android SDK.
  pause
  exit /b 1
)

adb start-server >nul 2>&1
adb wait-for-device

set PHONE=
for /f "skip=1 tokens=1,2" %%A in ('adb devices') do (
  echo %%A | findstr /b /c:"emulator-" >nul
  if errorlevel 1 (
    if "%%B"=="device" (
      if not defined PHONE set PHONE=%%A
    )
  )
)

if not defined PHONE (
  echo No USB phone found. adb devices:
  adb devices -l
  echo.
  echo If the phone is listed as unauthorized, unlock it and allow USB debugging.
  echo Unplug/replug the cable if it stays blank.
  pause
  exit /b 1
)

echo Using phone: %PHONE%
echo Forwarding Metro 8081 / debug ingest 7349 over USB...
adb -s %PHONE% reverse tcp:8081 tcp:8081
adb -s %PHONE% reverse tcp:8082 tcp:8082
adb -s %PHONE% reverse tcp:7349 tcp:7349
adb -s %PHONE% reverse tcp:3000 tcp:3000 >nul 2>&1

set APK=android\app\build\outputs\apk\debug\app-debug.apk
if exist "%APK%" (
  echo Installing existing debug APK...
  adb -s %PHONE% install -r "%APK%"
) else (
  echo No debug APK yet. Building and installing with Expo...
  call npx expo run:android --device %PHONE%
  if errorlevel 1 (
    echo Install failed.
    pause
    exit /b 1
  )
)

echo Launching NozyWallet...
adb -s %PHONE% shell am force-stop com.leoninedao.nozywallet
adb -s %PHONE% shell am start -n com.leoninedao.nozywallet/.MainActivity -a android.intent.action.MAIN -c android.intent.category.LAUNCHER

echo.
echo If you see Expo's development-build screen, Metro is not reachable over USB.
echo Metro on this PC must be running on port 8081 (this script starts it if it is not).
echo.

powershell -NoProfile -Command "try { (Invoke-WebRequest -Uri http://127.0.0.1:8081/status -UseBasicParsing -TimeoutSec 2).StatusCode } catch { 0 }" | findstr /r "200" >nul
if %ERRORLEVEL%==0 (
  echo Metro already running on 8081. App should load from USB reverse.
  pause
  exit /b 0
)

call npx expo start --port 8081 --offline
pause
