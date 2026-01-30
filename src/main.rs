mod core;
mod io;
mod models;
mod parser;
mod ui;

use core::planner::{self, UpdateMode};
use io::{command, file, terminal};
use parser::toml as toml_parser;
use std::path::PathBuf;
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
        match terminal::run_tui_with_scan(&config, has_paru) {
            Ok((Some(UIEvent::Reload), _)) => {
                // Reload: restart scan, do not save config
            },
            Ok((Some(event), final_state)) => {
                // Terminating event: save config if changed, then execute
                save_config_if_changed(&config_path, &config, &final_state);

                // Get all packages from final state
                let all_packages: Vec<models::package::Package> = final_state
                    .packages
                    .iter()
                    .map(|item| item.package.clone())
                    .collect();

                match event {
                    UIEvent::UpdateEntireSystem => {
                        let ignored = final_state.get_ignored_packages();
                        execute_update(UpdateMode::EntireSystem, all_packages, ignored, &config);
                    },
                    UIEvent::UpdateOfficialOnly => {
                        let ignored = final_state.get_ignored_packages();
                        execute_update(UpdateMode::OfficialOnly, all_packages, ignored, &config);
                    },
                    UIEvent::Quit => {},
                    UIEvent::Reload => {
                        panic!("DESIGN VIOLATION: UIEvent::Reload must be handled by the outer loop (Ok((Some(UIEvent::Reload), _)))")
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
