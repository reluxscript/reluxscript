# Uninstall ReluxScript Test Watcher Service
# Run as Administrator

$ErrorActionPreference = "Stop"

Write-Host "Uninstalling ReluxScript Test Watcher Service..." -ForegroundColor Cyan

$ServiceName = "ReluxScriptTestWatcher"

# Check if service exists
$service = Get-Service -Name $ServiceName -ErrorAction SilentlyContinue

if (-not $service) {
    Write-Host "Service not found. Nothing to uninstall." -ForegroundColor Yellow
    exit 0
}

# Stop the service if running
if ($service.Status -eq 'Running') {
    Write-Host "Stopping service..." -ForegroundColor Yellow
    Stop-Service -Name $ServiceName -Force
    Start-Sleep -Seconds 2
}

# Delete the service
Write-Host "Removing service..." -ForegroundColor Yellow
sc.exe delete $ServiceName

if ($LASTEXITCODE -eq 0) {
    Write-Host "✓ Service uninstalled successfully!" -ForegroundColor Green
} else {
    Write-Host "Failed to uninstall service!" -ForegroundColor Red
    exit 1
}
