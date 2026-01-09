@echo off
cd /d "%~dp0"
echo Starting Tauri development server...
echo Working directory: %CD%
cargo tauri dev
pause
