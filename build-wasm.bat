@echo off
echo Building ReluxScript WASM module for playground...
echo.

cd source

echo Running wasm-pack build...
wasm-pack build --target web --features wasm,codegen

if %ERRORLEVEL% EQU 0 (
    echo.
    echo ✓ WASM module built successfully!
    echo Output: site/pkg/
    echo.
    echo You can now run the playground:
    echo   cd site
    echo   npm run dev
) else (
    echo.
    echo ✗ Build failed with error code %ERRORLEVEL%
)

echo.
echo Press any key to exit...
pause
