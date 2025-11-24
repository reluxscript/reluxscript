@echo off
REM Run all enhancement tests to verify they fail (TDD red phase)
REM Each test should fail until the corresponding feature is implemented

setlocal enabledelayedexpansion

set SCRIPT_DIR=%~dp0
set RUSTSCRIPT_DIR=%SCRIPT_DIR%..\..

echo ================================
echo ReluxScript Enhancement Tests
echo ================================
echo.
echo Running TDD red phase - all tests should FAIL
echo.

set PASSED=0
set FAILED=0

REM Test 1: Nested Traverse
echo Testing: nested_traverse
echo ----------------------------------------
cargo run --manifest-path "%RUSTSCRIPT_DIR%\Cargo.toml" -- check "%SCRIPT_DIR%nested_traverse.rsc" 2>&1
if %ERRORLEVEL% EQU 0 (
    echo   UNEXPECTED PASS - Feature may already be implemented!
    set /a PASSED+=1
) else (
    echo   EXPECTED FAIL - Feature not yet implemented
    set /a FAILED+=1
)
echo.

REM Test 2: Mutable Refs
echo Testing: mutable_refs
echo ----------------------------------------
cargo run --manifest-path "%RUSTSCRIPT_DIR%\Cargo.toml" -- check "%SCRIPT_DIR%mutable_refs.rsc" 2>&1
if %ERRORLEVEL% EQU 0 (
    echo   UNEXPECTED PASS - Feature may already be implemented!
    set /a PASSED+=1
) else (
    echo   EXPECTED FAIL - Feature not yet implemented
    set /a FAILED+=1
)
echo.

REM Test 3: HashMap/HashSet
echo Testing: hashmap_hashset
echo ----------------------------------------
cargo run --manifest-path "%RUSTSCRIPT_DIR%\Cargo.toml" -- check "%SCRIPT_DIR%hashmap_hashset.rsc" 2>&1
if %ERRORLEVEL% EQU 0 (
    echo   UNEXPECTED PASS - Feature may already be implemented!
    set /a PASSED+=1
) else (
    echo   EXPECTED FAIL - Feature not yet implemented
    set /a FAILED+=1
)
echo.

REM Test 4: String Formatting
echo Testing: string_formatting
echo ----------------------------------------
cargo run --manifest-path "%RUSTSCRIPT_DIR%\Cargo.toml" -- check "%SCRIPT_DIR%string_formatting.rsc" 2>&1
if %ERRORLEVEL% EQU 0 (
    echo   UNEXPECTED PASS - Feature may already be implemented!
    set /a PASSED+=1
) else (
    echo   EXPECTED FAIL - Feature not yet implemented
    set /a FAILED+=1
)
echo.

REM Test 5: File I/O
echo Testing: file_io
echo ----------------------------------------
cargo run --manifest-path "%RUSTSCRIPT_DIR%\Cargo.toml" -- check "%SCRIPT_DIR%file_io.rsc" 2>&1
if %ERRORLEVEL% EQU 0 (
    echo   UNEXPECTED PASS - Feature may already be implemented!
    set /a PASSED+=1
) else (
    echo   EXPECTED FAIL - Feature not yet implemented
    set /a FAILED+=1
)
echo.

REM Test 6: JSON Serialization
echo Testing: json_serialization
echo ----------------------------------------
cargo run --manifest-path "%RUSTSCRIPT_DIR%\Cargo.toml" -- check "%SCRIPT_DIR%json_serialization.rsc" 2>&1
if %ERRORLEVEL% EQU 0 (
    echo   UNEXPECTED PASS - Feature may already be implemented!
    set /a PASSED+=1
) else (
    echo   EXPECTED FAIL - Feature not yet implemented
    set /a FAILED+=1
)
echo.

echo ================================
echo Summary
echo ================================
echo Expected failures (features needed): %FAILED%
echo Unexpected passes: %PASSED%
echo.

if %FAILED% EQU 6 (
    echo All tests failed as expected - TDD red phase complete!
    echo Now implement features to make tests pass.
) else (
    echo Some tests passed unexpectedly - check implementation status.
)

endlocal
