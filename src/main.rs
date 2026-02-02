mod app;
mod core;
mod io;
mod models;
mod parser;
mod ui;

use io::{command, file, terminal};
use parser::toml as toml_parser;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use ui::app::UIEvent;

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
        let scan_handle = app::start_scan_thread(tx, has_paru, Arc::clone(&cancel_flag));

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
                    app::save_config_if_changed(&config_path, &config, &final_state);
                }

                // Get all packages from final state
                let all_packages: Vec<models::package::Package> = final_state
                    .packages
                    .iter()
                    .map(|item| item.package.clone())
                    .collect();

                match event {
                    UIEvent::UpdateEntireSystem => {
                        app::handle_update(
                            &mut final_state,
                            all_packages,
                            &config,
                            core::planner::UpdateMode::EntireSystem,
                        );
                    }
                    UIEvent::UpdateOfficialOnly => {
                        app::handle_update(
                            &mut final_state,
                            all_packages,
                            &config,
                            core::planner::UpdateMode::OfficialOnly,
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
