mod core;
mod io;
mod models;
mod parser;
mod ui;

use core::planner::{self, UpdateMode};
use io::{command, file, terminal};
use parser::{pacman, toml as toml_parser};
use std::path::PathBuf;
use ui::app::UIEvent;

fn handle_update(
    final_state: &mut ui::app::AppState,
    all_packages: Vec<models::package::Package>,
    config: &models::config::Config,
    mode: UpdateMode,
) {
    let ignored = final_state.get_ignored_packages();

    match check_and_confirm_dependencies(final_state, &all_packages, &ignored) {
        Ok(true) => {
            execute_update(mode, all_packages, ignored, config);
        },
        Ok(false) => {
            // User cancelled, do nothing
        },
        Err(e) => {
            eprintln!("Failed to check dependencies: {e}");
        },
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
        match terminal::run_tui_with_scan(&config, has_paru) {
            Ok((Some(UIEvent::Reload), _)) => {
                // Reload: restart scan, do not save config
            },
            Ok((Some(event), mut final_state)) => {
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
                        handle_update(&mut final_state, all_packages, &config, UpdateMode::EntireSystem);
                    },
                    UIEvent::UpdateOfficialOnly => {
                        handle_update(&mut final_state, all_packages, &config, UpdateMode::OfficialOnly);
                    },
                    UIEvent::Quit => {},
                    UIEvent::Reload => {
                        panic!(
                            "DESIGN VIOLATION: UIEvent::Reload must be handled by the outer loop (Ok((Some(UIEvent::Reload), _)))"
                        )
                    },
                }
                break;
            },
            Ok((None, _)) => break,
            Err(e) => {
                eprintln!("TUI error: {e}");
                break;
            },
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
            },
            Err(e) => {
                eprintln!("Warning: Could not serialize config: {e:?}");
            },
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
                _ => Ok(false), // User cancelled or quit
            }
        },
        Err(warnings) => {
            for warning in warnings {
                eprintln!("Dependency check warning: {warning}");
            }
            Ok(false) // Don't proceed if dependency check failed
        },
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
        },
        Err(e) => {
            eprintln!("\n✗ Failed to execute update command: {e}");
        },
    }
}
