mod core;
mod io;
mod models;
mod parser;
mod ui;

use core::planner::{self, UpdateMode};
use io::{command, file, terminal};
use parser::{pacman, paru, toml as toml_parser};
use std::path::PathBuf;
use ui::app::{AppState, UIEvent};

fn main() {
    println!("par_tui - Arch Linux Update Manager\n");

    // Load config
    let config_home = std::env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| {
        PathBuf::from(std::env::var("HOME").unwrap_or_default())
            .join(".config")
            .to_string_lossy()
            .to_string()
    });
    let config_path = PathBuf::from(config_home).join("partui/config.toml");

    let config = if let Ok(content) = file::read_config(&config_path) { match toml_parser::parse_config(&content) {
        Ok(cfg) => {
            println!("Config loaded from: {}", config_path.display());
            cfg
        }
        Err(e) => {
            println!("Config parse error: {e:?}, using defaults");
            models::config::Config::default()
        }
    } } else {
        println!("No config found, using defaults");
        models::config::Config::default()
    };

    // Check for required commands
    println!("Checking for required commands...");
    let has_checkupdates = command::check_command_exists("checkupdates");
    let has_paru = command::check_command_exists("paru");

    if !has_checkupdates {
        eprintln!("\nError: checkupdates not found. Install pacman-contrib.");
        return;
    }

    // Scan for updates
    println!("Scanning for updates...\n");
    let mut all_packages = Vec::new();

    match command::run_checkupdates() {
        Ok(output) => {
            let packages = pacman::parse_checkupdates_output(&output);
            println!("Found {} official updates", packages.len());
            all_packages.extend(packages);
        }
        Err(e) => {
            eprintln!("Warning: Could not scan official updates: {e:?}");
        }
    }

    if has_paru {
        match command::run_paru_query_aur() {
            Ok(output) => {
                let packages = paru::parse_paru_output(&output);
                println!("Found {} AUR updates", packages.len());
                all_packages.extend(packages);
            }
            Err(e) => {
                eprintln!("Warning: Could not scan AUR updates: {e:?}");
            }
        }
    }

    if all_packages.is_empty() {
        println!("\nSystem is up to date!");
        return;
    }

    // Launch TUI
    println!("\nLaunching TUI...");
    let state = AppState::new(all_packages.clone(), &config.exclude.permanent);

    match terminal::run_tui(state) {
        Ok((Some(event), final_state)) => match event {
            UIEvent::UpdateEntireSystem => {
                let ignored = final_state.get_ignored_packages();
                execute_update(UpdateMode::EntireSystem, all_packages, ignored, &config);
            }
            UIEvent::UpdateOfficialOnly => {
                let ignored = final_state.get_ignored_packages();
                execute_update(UpdateMode::OfficialOnly, all_packages, ignored, &config);
            }
            UIEvent::Quit => {
                println!("\nExiting without updates.");
            }
            UIEvent::OpenLink(_) => {
                println!("\nLink opening not yet implemented.");
            }
        },
        Ok((None, _)) => {
            println!("\nNo action taken.");
        }
        Err(e) => {
            eprintln!("\nTUI error: {e}");
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

    // Dry-run check
    if std::env::var("PAR_TUI_DRY_RUN").is_ok() {
        println!("\n[DRY RUN] Would execute: {}", cmd.join(" "));
        println!("Ignored packages: {:?}", plan.ignore_list);
        return;
    }

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
