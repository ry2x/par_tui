mod core;
mod io;
mod models;
mod parser;
mod ui;

use core::planner::{self, UpdateMode};
use io::{command, file, terminal};
use models::package::Package;
use parser::{pacman, paru, toml as toml_parser};
use std::path::PathBuf;
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::thread::{self, JoinHandle};
use ui::app::UIEvent;

/// Message types for scan thread communication
pub enum ScanMessage {
    Progress(String),
    ScanWarning(String),
    Complete(Vec<Package>),
}

/// Scan failure markers for warning messages
pub const OFFICIAL_SCAN_FAILURE_MARKER: &str = "Official";
pub const AUR_SCAN_FAILURE_MARKER: &str = "AUR";

/// Starts background thread to scan for package updates.
///
/// Sends progress updates via channel and handles cancellation.
fn start_scan_thread(
    tx: Sender<ScanMessage>,
    has_paru: bool,
    cancel_flag: Arc<AtomicBool>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        // Helper macro to send message and return early if channel is closed
        macro_rules! send_or_return {
            ($msg:expr) => {
                if tx.send($msg).is_err() {
                    return;
                }
            };
        }

        let mut all_packages = Vec::new();
        let mut official_failed = false;
        let mut aur_failed = false;

        // Scan official packages
        if cancel_flag.load(Ordering::Relaxed) {
            return;
        }

        send_or_return!(ScanMessage::Progress(
            "Scanning official repositories...".to_string()
        ));

        let tx_clone = tx.clone();
        match command::run_checkupdates_with_callback(|attempt, max| {
            let _ = tx_clone.send(ScanMessage::Progress(format!(
                "Retrying checkupdates (attempt {attempt}/{max})"
            )));
        }) {
            Ok(output) => {
                let packages = pacman::parse_checkupdates_output(&output);
                let count = packages.len();
                send_or_return!(ScanMessage::Progress(format!(
                    "Found {} official update{}",
                    count,
                    if count == 1 { "" } else { "s" }
                )));
                all_packages.extend(packages);
            }
            Err(e) => {
                official_failed = true;
                send_or_return!(ScanMessage::Progress(format!(
                    "Warning: Could not scan official repos: {e:?}"
                )));
            }
        }

        // Scan AUR packages
        if has_paru && !cancel_flag.load(Ordering::Relaxed) {
            send_or_return!(ScanMessage::Progress(
                "Scanning AUR packages...".to_string()
            ));

            match command::run_paru_query_aur() {
                Ok(output) => {
                    let packages = paru::parse_paru_output(&output);
                    let count = packages.len();
                    send_or_return!(ScanMessage::Progress(format!(
                        "Found {} AUR update{}",
                        count,
                        if count == 1 { "" } else { "s" }
                    )));
                    all_packages.extend(packages);
                }
                Err(e) => {
                    aur_failed = true;
                    send_or_return!(ScanMessage::Progress(format!(
                        "Warning: Could not scan AUR packages: {e:?}"
                    )));
                }
            }
        }

        // Check if cancelled before sending final messages
        if cancel_flag.load(Ordering::Relaxed) {
            return;
        }

        // Final status message
        let total = all_packages.len();
        send_or_return!(ScanMessage::Progress(format!(
            "Scan complete. Total: {} update{}",
            total,
            if total == 1 { "" } else { "s" }
        )));

        // Send warning about scan failures
        if official_failed || aur_failed {
            let mut failed_sources = Vec::new();
            if official_failed {
                failed_sources.push(OFFICIAL_SCAN_FAILURE_MARKER);
            }
            if aur_failed {
                failed_sources.push(AUR_SCAN_FAILURE_MARKER);
            }
            send_or_return!(ScanMessage::ScanWarning(format!(
                "{} scan failed",
                failed_sources.join(" & ")
            )));
        }

        send_or_return!(ScanMessage::Complete(all_packages));
    })
}

fn handle_update(
    final_state: &mut ui::app::AppState,
    all_packages: Vec<models::package::Package>,
    config: &models::config::Config,
    mode: UpdateMode,
) -> bool {
    let ignored = final_state.get_ignored_packages();

    match check_and_confirm_dependencies(final_state, &all_packages, &ignored) {
        Ok(true) => {
            execute_update(mode, all_packages, ignored, config);
            true
        }
        Ok(false) => {
            // User cancelled, do nothing
            false
        }
        Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {
            // User quit during confirmation
            false
        }
        Err(e) => {
            eprintln!("Failed to check dependencies: {e}");
            false
        }
    }
}

fn main() {
    // Load config
    let config_home = std::env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| {
        PathBuf::from(std::env::var("HOME").unwrap_or_default())
            .join(".config")
            .to_string_lossy()
            .to_string()
    });
    let config_path = PathBuf::from(config_home).join("partui/config.toml");

    let config = if let Ok(content) = file::read_config(&config_path) {
        toml_parser::parse_config(&content).unwrap_or_default()
    } else {
        models::config::Config::default()
    };

    // Check for required commands
    let has_checkupdates = command::check_command_exists("checkupdates");
    let has_paru = command::check_command_exists("paru");

    if !has_checkupdates {
        eprintln!("Error: checkupdates not found. Install pacman-contrib.");
        return;
    }

    // Launch TUI with async scanning (loop for reload)
    loop {
        // Setup scan thread
        let (tx, rx) = mpsc::channel();
        let cancel_flag = Arc::new(AtomicBool::new(false));
        let scan_handle = start_scan_thread(tx, has_paru, Arc::clone(&cancel_flag));

        match terminal::run_tui_with_scan(&config, rx, cancel_flag.clone()) {
            Ok((Some(UIEvent::Reload), _)) => {
                // Signal thread to stop
                cancel_flag.store(true, Ordering::Relaxed);
                // Wait for thread to complete
                if scan_handle.join().is_err() {
                    eprintln!("Warning: Scan thread panicked during reload.");
                }
                // Reload: restart scan, do not save config
            }
            Ok((Some(event), mut final_state)) => {
                // Signal thread to stop
                cancel_flag.store(true, Ordering::Relaxed);
                // Wait for thread to complete
                if scan_handle.join().is_err() {
                    eprintln!("Warning: Scan thread panicked during execution.");
                }

                // Terminating event: save config if changed, then execute
                // Skip saving if state is not ready (e.g., quit during scan)
                if final_state.is_ready() {
                    save_config_if_changed(&config_path, &config, &final_state);
                }

                // Get all packages from final state
                let all_packages: Vec<models::package::Package> = final_state
                    .packages
                    .iter()
                    .map(|item| item.package.clone())
                    .collect();

                match event {
                    UIEvent::UpdateEntireSystem => {
                        handle_update(
                            &mut final_state,
                            all_packages,
                            &config,
                            UpdateMode::EntireSystem,
                        );
                    }
                    UIEvent::UpdateOfficialOnly => {
                        handle_update(
                            &mut final_state,
                            all_packages,
                            &config,
                            UpdateMode::OfficialOnly,
                        );
                    }
                    UIEvent::Quit | UIEvent::Reload => {}
                }
                break;
            }
            Ok((None, _)) => {
                // Signal thread to stop
                cancel_flag.store(true, Ordering::Relaxed);
                // Wait for thread to complete
                if scan_handle.join().is_err() {
                    eprintln!("Warning: Scan thread panicked during quit.");
                }
                break;
            }
            Err(e) => {
                // Signal thread to stop
                cancel_flag.store(true, Ordering::Relaxed);
                // Wait for thread to complete
                let _ = scan_handle.join();
                eprintln!("TUI error: {e}");
                break;
            }
        }
    }
}

fn save_config_if_changed(
    config_path: &std::path::Path,
    config: &models::config::Config,
    final_state: &ui::app::AppState,
) {
    let new_permanent = final_state.get_permanent_excludes();
    if new_permanent != config.exclude.permanent {
        let mut updated_config = config.clone();
        updated_config.exclude.permanent = new_permanent;
        match toml_parser::serialize_config(&updated_config) {
            Ok(content) => {
                if let Err(e) = file::write_config(config_path, &content) {
                    eprintln!("Warning: Could not save config: {e:?}");
                } else {
                    println!("Permanent excludes saved to config.");
                }
            }
            Err(e) => {
                eprintln!("Warning: Could not serialize config: {e:?}");
            }
        }
    }
}

fn check_and_confirm_dependencies(
    state: &mut ui::app::AppState,
    all_packages: &[models::package::Package],
    ignored: &[String],
) -> std::io::Result<bool> {
    // Perform dependency check (orchestration: main.rs calls core and parser)
    match core::dependency::check_conflicts(all_packages, ignored, |pkg| {
        state.get_or_fetch_required_by(pkg, || {
            command::get_package_required_by(pkg)
                .map(|output| pacman::parse_required_by(&output))
                .map_err(|e| e.to_string())
        })
    }) {
        Ok(conflicts) => {
            if conflicts.is_empty() {
                // No conflicts, proceed
                return Ok(true);
            }

            // Conflicts found, show modal for user decision
            state.set_dependency_conflicts(conflicts);
            state.show_dependency_warning = true;

            // Re-enter TUI for confirmation
            match terminal::run_tui_for_confirmation(state)? {
                Some(UIEvent::UpdateEntireSystem | UIEvent::UpdateOfficialOnly) => Ok(true),
                Some(UIEvent::Quit) => Err(std::io::Error::new(
                    std::io::ErrorKind::Interrupted,
                    "User quit during dependency confirmation",
                )),
                _ => Ok(false), // User cancelled
            }
        }
        Err(warnings) => {
            for warning in warnings {
                eprintln!("Dependency check warning: {warning}");
            }
            Ok(false) // Don't proceed if dependency check failed
        }
    }
}

fn execute_update(
    mode: UpdateMode,
    packages: Vec<models::package::Package>,
    ignored: Vec<String>,
    config: &models::config::Config,
) {
    let plan = planner::create_plan(mode, packages, ignored);
    let cmd = plan.build_command(config);

    println!("\n{}", "=".repeat(60));
    println!("Executing: {}", cmd.join(" "));
    println!("{}", "=".repeat(60));
    println!();

    match plan.execute(config) {
        Ok(status) => {
            if status.success() {
                println!("\n✓ Update completed successfully!");
            } else {
                eprintln!(
                    "\n✗ Update failed with exit code: {}",
                    status.code().unwrap_or(-1)
                );
            }
        }
        Err(e) => {
            eprintln!("\n✗ Failed to execute update command: {e}");
        }
    }
}
