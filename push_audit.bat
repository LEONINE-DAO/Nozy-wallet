@echo off
echo === Pushing Security Audit Files ===
echo.

echo 1. Adding files...
git add -f SELF_AUDIT_RESULTS.md SELF_SECURITY_AUDIT_GUIDE.md SECURITY_AUDIT_ANNOUNCEMENT.md run_self_audit.ps1 .gitignore
if errorlevel 1 (
    echo ERROR: Failed to add files
    pause
    exit /b 1
)
echo Files added successfully
echo.

echo 2. Checking status...
git status --short
echo.

echo 3. Committing...
git commit -m "Add security self-audit results and community announcement"
if errorlevel 1 (
    echo ERROR: Failed to commit
    pause
    exit /b 1
)
echo Commit successful
echo.

echo 4. Pushing to GitHub...
git push origin master
if errorlevel 1 (
    echo ERROR: Failed to push
    echo You may need to pull first: git pull origin master
    pause
    exit /b 1
)
echo.
echo === SUCCESS ===
echo Files pushed to GitHub!
echo.
pause
