# Install ReluxScript Test Watcher Service
# Run as Administrator

param(
    [switch]$Build = $true
)

$ErrorActionPreference = "Stop"

Write-Host "Installing ReluxScript Test Watcher Service..." -ForegroundColor Cyan

# Build the service if requested
if ($Build) {
    Write-Host "Building service..." -ForegroundColor Yellow
    cargo build --release
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Build failed!" -ForegroundColor Red
        exit 1
    }
}

$ServiceName = "ReluxScriptTestWatcher"
$BinaryPath = Join-Path $PSScriptRoot "target\release\reluxscript-test-watcher.exe"

# Check if binary exists
if (-not (Test-Path $BinaryPath)) {
    Write-Host "Binary not found at: $BinaryPath" -ForegroundColor Red
    Write-Host "Run with -Build switch or build manually first" -ForegroundColor Yellow
    exit 1
}

# Check if service already exists
$existingService = Get-Service -Name $ServiceName -ErrorAction SilentlyContinue

if ($existingService) {
    Write-Host "Service already exists. Stopping and removing..." -ForegroundColor Yellow

    if ($existingService.Status -eq 'Running') {
        Stop-Service -Name $ServiceName -Force
        Start-Sleep -Seconds 2
    }

    sc.exe delete $ServiceName
    Start-Sleep -Seconds 2
}

# Create the service
Write-Host "Creating Windows service..." -ForegroundColor Yellow
sc.exe create $ServiceName binPath= $BinaryPath start= auto DisplayName= "ReluxScript Test Watcher"

if ($LASTEXITCODE -ne 0) {
    Write-Host "Failed to create service!" -ForegroundColor Red
    exit 1
}

# Start the service
Write-Host "Starting service..." -ForegroundColor Yellow
Start-Service -Name $ServiceName

# Check status
$service = Get-Service -Name $ServiceName
if ($service.Status -eq 'Running') {
    Write-Host "✓ Service installed and started successfully!" -ForegroundColor Green
    Write-Host ""
    Write-Host "The service will now monitor your ReluxScript repository and send notifications when tests run." -ForegroundColor Cyan
    Write-Host ""
    Write-Host "To view logs, check Windows Event Viewer or run:" -ForegroundColor Gray
    Write-Host "  Get-EventLog -LogName Application -Source $ServiceName -Newest 50" -ForegroundColor Gray
} else {
    Write-Host "Service created but failed to start. Status: $($service.Status)" -ForegroundColor Red
    Write-Host "Check Windows Event Viewer for details" -ForegroundColor Yellow
}
