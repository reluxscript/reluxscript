# ReluxScript Test Watcher

A Windows service that monitors your ReluxScript repository for changes and automatically runs tests, sending Windows 11 toast notifications with results.

## Features

- **Automatic Test Execution**: Watches `.rs`, `.lux`, and `.toml` files for changes
- **Smart Debouncing**: Waits 2 seconds after the last change before running tests (prevents spam)
- **Windows Notifications**: Sends toast notifications with test results
  - ✓ Green notifications for passing tests
  - ✗ Red notifications with failure details
- **Background Service**: Runs as a Windows service, starts automatically on boot

## Installation

### Prerequisites

- Windows 11 (or Windows 10 with toast notification support)
- Rust toolchain installed
- Administrator privileges

### Install as Windows Service

1. Open PowerShell as Administrator
2. Navigate to this directory
3. Run the installer:

```powershell
.\install-service.ps1
```

The service will:
- Build the release binary
- Install as a Windows service
- Start automatically
- Run on system startup

### Test in Console Mode First (Recommended)

Before installing as a service, test in console mode:

```powershell
.\run-console.ps1
```

This runs the watcher in your terminal so you can see the output directly.

## Usage

Once installed, the service runs automatically. It will:

1. Monitor `J:\projects\relux\reluxscript\source\` for changes
2. When you save a `.rs`, `.lux`, or `.toml` file
3. Wait 2 seconds for additional changes (debounce)
4. Run `cargo test` in the source directory
5. Send a Windows notification with results

## Notifications

You'll receive three types of notifications:

1. **Service Started**: When the watcher starts monitoring
2. **Tests Passed ✓**: All tests passed successfully
3. **Tests Failed ✗**: Shows which tests failed

Click on failure notifications to see details (if configured).

## Configuration

Edit `src/main.rs` to customize:

```rust
WatcherConfig {
    repo_path: PathBuf::from("J:\\projects\\relux\\reluxscript"),
    debounce_ms: 2000, // Wait time in milliseconds
}
```

After changing configuration, rebuild and reinstall:

```powershell
.\install-service.ps1
```

## Uninstallation

```powershell
.\uninstall-service.ps1
```

## Troubleshooting

### Service won't start

Check Event Viewer:
- Windows Logs → Application
- Look for errors from "ReluxScriptTestWatcher"

### No notifications appearing

1. Check Windows notification settings
2. Ensure notifications are enabled for PowerShell
3. Run in console mode to see if tests are actually running

### Tests not running

1. Verify the repo path in configuration
2. Check file permissions
3. Ensure `cargo` is in system PATH (not just user PATH)

## Development

### Build

```bash
cargo build
```

### Run in debug mode

```bash
cargo run
```

### View logs

When running as a service, logs go to Windows Event Viewer.

When running in console mode with:

```powershell
$env:RUST_LOG = "info"
cargo run
```

## Architecture

```
File System Watcher (notify crate)
    ↓
Detects .rs/.lux/.toml changes
    ↓
Debounce (2 second delay)
    ↓
Execute: cargo test
    ↓
Parse output
    ↓
Windows Toast Notification (winrt-notification)
```

## Dependencies

- `windows-service`: Windows service framework
- `notify`: Cross-platform file system watcher
- `winrt-notification`: Windows 10/11 toast notifications
- `tokio`: Async runtime
- `anyhow`: Error handling
- `log` + `env_logger`: Logging

## License

Same as ReluxScript project
