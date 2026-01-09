@echo off
cd /d "%~dp0"
echo Pushing zeaking code to https://github.com/Lowo88/Zeaking
echo.

echo Step 1: Ensuring remote is configured...
git remote remove zeaking 2>nul
git remote add zeaking https://github.com/Lowo88/Zeaking.git
git remote -v
echo.

echo Step 2: Ensuring zeaking directory is committed...
git add zeaking/
git commit -m "Add zeaking indexing library" 2>nul
echo.

echo Step 3: Creating subtree branch...
git subtree split --prefix=zeaking -b zeaking-export
if errorlevel 1 (
    echo ERROR: Failed to create subtree branch
    pause
    exit /b 1
)
echo.

echo Step 4: Pushing to master branch...
git push zeaking zeaking-export:master --force
if errorlevel 1 (
    echo Master branch failed, trying main branch...
    git push zeaking zeaking-export:main --force
    if errorlevel 1 (
        echo ERROR: Failed to push to repository
        echo.
        echo Make sure:
        echo   1. The repository exists at https://github.com/Lowo88/Zeaking
        echo   2. You have push access
        echo   3. Your git credentials are configured
        pause
        exit /b 1
    )
)
echo.
echo SUCCESS! Check https://github.com/Lowo88/Zeaking to verify.
pause
