@echo off
title NozyPixel emulator (clean ADB)
set SDK=C:\Users\User\AppData\Local\Android\Sdk
if exist "%SDK%\emulator\emulator.exe" goto :haveSdk
set SDK=C:\Android\Sdk
:haveSdk
set EMU=%SDK%\emulator\emulator.exe
set ADB=%SDK%\platform-tools\adb.exe
set LOG=%~dp0emulator-console.log

if not exist "%EMU%" (
  echo emulator.exe not found. Install Android SDK emulator tools.
  pause
  exit /b 1
)

echo Starting NozyPixel with -delay-adb.
echo Emulator console (Play Services dumpsys spam) goes to:
echo   %LOG%
echo Do not run adb kill-server while the emulator is up.
echo Do not use -wipe-data unless you mean to erase the AVD.
echo.

REM -debug-no-adb* keeps qemu from printing GMS dumpsys / protocol faults to this window.
REM stdout/stderr are also logged so this cmd window stays readable.
start "NozyPixel" /min cmd /c ""%EMU%" -avd NozyPixel -delay-adb -netdelay none -netspeed full -debug-no-adb -debug-no-adbclient -debug-no-adbserver > "%LOG%" 2>&1"

echo Waiting for adb device...
"%ADB%" wait-for-device
:waitBoot
"%ADB%" shell getprop sys.boot_completed 2>nul | findstr /r /c:"^1" >nul
if errorlevel 1 (
  timeout /t 2 /nobreak >nul
  goto waitBoot
)
"%ADB%" reverse tcp:8081 tcp:8081
"%ADB%" reverse tcp:8082 tcp:8082
"%ADB%" reverse tcp:7349 tcp:7349
echo Emulator is online. ADB reverse is set for Metro 8081 and debug 7349.
echo.
pause
