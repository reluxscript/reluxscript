use anyhow::Result;
use log::{error, info};
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::process::Command;
use std::sync::mpsc::channel;
use std::time::Duration;
use winrt_notification::{Duration as ToastDuration, Sound, Toast};

#[cfg(windows)]
use windows_service::{
    define_windows_service,
    service::{
        ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
        ServiceType,
    },
    service_control_handler::{self, ServiceControlHandlerResult},
    service_dispatcher,
};

const SERVICE_NAME: &str = "ReluxScriptTestWatcher";
const SERVICE_TYPE: ServiceType = ServiceType::OWN_PROCESS;

// Configuration for the watcher
struct WatcherConfig {
    repo_path: PathBuf,
    debounce_ms: u64,
}

impl Default for WatcherConfig {
    fn default() -> Self {
        Self {
            repo_path: PathBuf::from("J:\\projects\\relux\\reluxscript"),
            debounce_ms: 2000, // Wait 2 seconds after last change before running tests
        }
    }
}

fn main() -> Result<()> {
    env_logger::init();

    #[cfg(windows)]
    {
        // Run as Windows service
        service_dispatcher::start(SERVICE_NAME, ffi_service_main)?;
    }

    #[cfg(not(windows))]
    {
        // Run in console mode for testing
        run_watcher()?;
    }

    Ok(())
}

#[cfg(windows)]
define_windows_service!(ffi_service_main, service_main);

#[cfg(windows)]
fn service_main(_arguments: Vec<std::ffi::OsString>) {
    if let Err(e) = run_service() {
        error!("Service error: {}", e);
    }
}

#[cfg(windows)]
fn run_service() -> Result<()> {
    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        match control_event {
            ServiceControl::Stop | ServiceControl::Interrogate => {
                ServiceControlHandlerResult::NoError
            }
            _ => ServiceControlHandlerResult::NotImplemented,
        }
    };

    let status_handle = service_control_handler::register(SERVICE_NAME, event_handler)?;

    // Tell Windows we're running
    status_handle.set_service_status(ServiceStatus {
        service_type: SERVICE_TYPE,
        current_state: ServiceState::Running,
        controls_accepted: ServiceControlAccept::STOP,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    })?;

    info!("ReluxScript Test Watcher service started");

    // Run the actual watcher
    if let Err(e) = run_watcher() {
        error!("Watcher error: {}", e);
    }

    // Tell Windows we're stopped
    status_handle.set_service_status(ServiceStatus {
        service_type: SERVICE_TYPE,
        current_state: ServiceState::Stopped,
        controls_accepted: ServiceControlAccept::empty(),
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    })?;

    Ok(())
}

fn run_watcher() -> Result<()> {
    let config = WatcherConfig::default();
    info!("Watching: {}", config.repo_path.display());

    let (tx, rx) = channel();

    let mut watcher = RecommendedWatcher::new(
        move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        },
        Config::default(),
    )?;

    // Watch the source directory
    let watch_path = config.repo_path.join("source");
    watcher.watch(&watch_path, RecursiveMode::Recursive)?;

    info!("File watcher started. Monitoring for changes...");
    send_notification("Test Watcher Started", "Monitoring for file changes", true);

    let mut last_event_time = std::time::Instant::now();
    let mut pending_test = false;

    loop {
        match rx.recv_timeout(Duration::from_millis(500)) {
            Ok(event) => {
                // Check if this is a relevant file change
                if is_relevant_change(&event) {
                    info!("Detected change: {:?}", event);
                    last_event_time = std::time::Instant::now();
                    pending_test = true;
                }
            }
            Err(_) => {
                // Timeout - check if we should run tests
                if pending_test && last_event_time.elapsed().as_millis() > config.debounce_ms as u128 {
                    pending_test = false;
                    run_tests(&config.repo_path);
                }
            }
        }
    }
}

fn is_relevant_change(event: &Event) -> bool {
    event.paths.iter().any(|path| {
        let ext = path.extension().and_then(|s| s.to_str());
        matches!(ext, Some("rs") | Some("lux") | Some("toml"))
    })
}

fn run_tests(repo_path: &PathBuf) {
    info!("Running tests...");
    send_notification("Running Tests", "Testing ReluxScript...", true);

    let source_path = repo_path.join("source");

    let output = Command::new("cargo")
        .arg("test")
        .current_dir(&source_path)
        .output();

    match output {
        Ok(result) => {
            let success = result.status.success();
            let stdout = String::from_utf8_lossy(&result.stdout);
            let stderr = String::from_utf8_lossy(&result.stderr);

            if success {
                info!("✓ Tests passed!");
                send_notification(
                    "Tests Passed ✓",
                    "All ReluxScript tests passed successfully",
                    true,
                );
            } else {
                error!("✗ Tests failed");
                error!("stdout: {}", stdout);
                error!("stderr: {}", stderr);

                // Parse failure count from output
                let failure_msg = parse_test_failures(&stdout, &stderr);
                send_notification("Tests Failed ✗", &failure_msg, false);
            }
        }
        Err(e) => {
            error!("Failed to run cargo test: {}", e);
            send_notification(
                "Test Error",
                &format!("Failed to run tests: {}", e),
                false,
            );
        }
    }
}

fn parse_test_failures(stdout: &str, stderr: &str) -> String {
    // Try to extract failure count from cargo output
    let output = format!("{}\n{}", stdout, stderr);

    for line in output.lines() {
        if line.contains("test result:") {
            return line.to_string();
        }
    }

    "Tests failed - check logs for details".to_string()
}

fn send_notification(title: &str, message: &str, success: bool) {
    let sound = if success {
        Sound::Default
    } else {
        Sound::SMS
    };

    let toast = Toast::new(Toast::POWERSHELL_APP_ID)
        .title(title)
        .text1(message)
        .sound(Some(sound))
        .duration(ToastDuration::Short);

    if let Err(e) = toast.show() {
        error!("Failed to show notification: {}", e);
    }
}
