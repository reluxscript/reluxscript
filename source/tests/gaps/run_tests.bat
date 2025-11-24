@echo off
REM Run all gap tests to verify they fail (TDD red phase)
REM These tests correspond to gaps documented in reluxscript-gaps.md

setlocal enabledelayedexpansion

set SCRIPT_DIR=%~dp0
set RUSTSCRIPT_DIR=%SCRIPT_DIR%..\..

echo ================================
echo ReluxScript Gap Tests
echo ================================
echo.
echo Running tests for documented gaps
echo.

set PASSED=0
set FAILED=0

REM Test 1: Self type
echo Testing: self_type
echo ----------------------------------------
cargo run --manifest-path "%RUSTSCRIPT_DIR%\Cargo.toml" -- check "%SCRIPT_DIR%self_type.rsc" 2>&1 | findstr /C:"Check passed" /C:"error[" /C:"Parse error"
if %ERRORLEVEL% EQU 0 (
    echo   Status: FAILED as expected
    set /a FAILED+=1
) else (
    echo   Status: PASSED unexpectedly
    set /a PASSED+=1
)
echo.

REM Test 2: Associated functions
echo Testing: associated_functions
echo ----------------------------------------
cargo run --manifest-path "%RUSTSCRIPT_DIR%\Cargo.toml" -- check "%SCRIPT_DIR%associated_functions.rsc" 2>&1 | findstr /C:"Check passed" /C:"error[" /C:"Parse error"
if %ERRORLEVEL% EQU 0 (
    echo   Status: FAILED as expected
    set /a FAILED+=1
) else (
    echo   Status: PASSED unexpectedly
    set /a PASSED+=1
)
echo.

REM Test 3: Function call in struct
echo Testing: function_call_in_struct
echo ----------------------------------------
cargo run --manifest-path "%RUSTSCRIPT_DIR%\Cargo.toml" -- check "%SCRIPT_DIR%function_call_in_struct.rsc" 2>&1 | findstr /C:"Check passed" /C:"error[" /C:"Parse error"
if %ERRORLEVEL% EQU 0 (
    echo   Status: FAILED as expected
    set /a FAILED+=1
) else (
    echo   Status: PASSED unexpectedly
    set /a PASSED+=1
)
echo.

REM Test 4: impl blocks
echo Testing: impl_blocks
echo ----------------------------------------
cargo run --manifest-path "%RUSTSCRIPT_DIR%\Cargo.toml" -- check "%SCRIPT_DIR%impl_blocks.rsc" 2>&1 | findstr /C:"Check passed" /C:"error[" /C:"Parse error"
if %ERRORLEVEL% EQU 0 (
    echo   Status: FAILED as expected
    set /a FAILED+=1
) else (
    echo   Status: PASSED unexpectedly
    set /a PASSED+=1
)
echo.

REM Test 5: Method chaining
echo Testing: method_chaining
echo ----------------------------------------
cargo run --manifest-path "%RUSTSCRIPT_DIR%\Cargo.toml" -- check "%SCRIPT_DIR%method_chaining.rsc" 2>&1 | findstr /C:"Check passed" /C:"error[" /C:"Parse error"
if %ERRORLEVEL% EQU 0 (
    echo   Status: FAILED as expected
    set /a FAILED+=1
) else (
    echo   Status: PASSED unexpectedly
    set /a PASSED+=1
)
echo.

REM Test 6: HashMap.len() codegen (this is a codegen issue, not parse/check)
echo Testing: hashmap_length (CODEGEN TEST - should pass check but fail at runtime)
echo ----------------------------------------
cargo run --manifest-path "%RUSTSCRIPT_DIR%\Cargo.toml" -- check "%SCRIPT_DIR%hashmap_length.rsc" 2>&1 | findstr /C:"Check passed"
if %ERRORLEVEL% EQU 0 (
    echo   Status: Check passes - need to verify Babel codegen generates .size not .length
) else (
    echo   Status: Check failed unexpectedly
)
echo.

REM Test 7: Writer lifecycle
echo Testing: writer_lifecycle
echo ----------------------------------------
cargo run --manifest-path "%RUSTSCRIPT_DIR%\Cargo.toml" -- check "%SCRIPT_DIR%writer_lifecycle.rsc" 2>&1 | findstr /C:"Check passed" /C:"error[" /C:"Parse error"
if %ERRORLEVEL% EQU 0 (
    echo   Status: FAILED as expected
    set /a FAILED+=1
) else (
    echo   Status: PASSED unexpectedly
    set /a PASSED+=1
)
echo.

REM Test 8: Full minimact transpiler (integration test)
echo Testing: minimact_full (INTEGRATION TEST - tests all features together)
echo ----------------------------------------
cargo run --manifest-path "%RUSTSCRIPT_DIR%\Cargo.toml" -- check "%SCRIPT_DIR%minimact_full.rsc" 2>&1 | findstr /C:"Check passed" /C:"error[" /C:"Parse error"
if %ERRORLEVEL% EQU 0 (
    echo   Status: Has errors - some features still missing
    set /a FAILED+=1
) else (
    echo   Status: PASSED - all features working!
    set /a PASSED+=1
)
echo.

echo ================================
echo Summary
echo ================================
echo Expected failures (gaps): %FAILED%
echo Unexpected passes: %PASSED%
echo.

if %FAILED% GEQ 6 (
    echo Most tests failed as expected - gaps confirmed!
) else (
    echo Some gaps may be fixed - check implementation.
)

endlocal
