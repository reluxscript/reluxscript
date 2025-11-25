@echo off
echo Installing Node.js dependencies for integration tests...
echo.

cd %~dp0
npm install

if %ERRORLEVEL% EQU 0 (
    echo.
    echo ✓ Dependencies installed successfully!
) else (
    echo.
    echo ✗ Installation failed with error code %ERRORLEVEL%
)

pause
