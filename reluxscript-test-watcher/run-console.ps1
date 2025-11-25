# Run the test watcher in console mode (not as service)
# Useful for development and testing

param(
    [switch]$Build = $true
)

$ErrorActionPreference = "Stop"

Write-Host "Running ReluxScript Test Watcher in console mode..." -ForegroundColor Cyan
Write-Host "Press Ctrl+C to stop" -ForegroundColor Gray
Write-Host ""

if ($Build) {
    Write-Host "Building..." -ForegroundColor Yellow
    cargo build
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Build failed!" -ForegroundColor Red
        exit 1
    }
}

# Set environment variable to run in console mode
$env:RUST_LOG = "info"

Write-Host "Starting watcher..." -ForegroundColor Green
Write-Host ""

# Run the executable
& ".\target\debug\reluxscript-test-watcher.exe"
