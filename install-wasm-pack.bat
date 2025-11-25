@echo off
echo Installing wasm-pack with Visual Studio 2022 environment...
echo.

call "C:\Program Files\Microsoft Visual Studio\2022\Community\Common7\Tools\VsDevCmd.bat" -arch=amd64 -host_arch=amd64

echo.
echo Running cargo install wasm-pack...
cargo install wasm-pack

echo.
echo Done! Press any key to exit...
pause
