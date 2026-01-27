mod core;
mod io;
mod models;
mod parser;
mod ui;

use core::{filter, planner};
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

    let config = match file::read_config(&config_path) {
        Ok(content) => match toml_parser::parse_config(&content) {
            Ok(cfg) => {
                println!("Config loaded from: {}", config_path.display());
                cfg
            }
            Err(e) => {
                println!("Config parse error: {:?}, using defaults", e);
                models::config::Config::default()
            }
        },
        Err(_) => {
            println!("No config found, using defaults");
            models::config::Config::default()
        }
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
            eprintln!("Warning: Could not scan official updates: {:?}", e);
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
                eprintln!("Warning: Could not scan AUR updates: {:?}", e);
            }
        }
    }

    if all_packages.is_empty() {
        println!("\nSystem is up to date!");
        return;
    }

    // Launch TUI
    println!("\nLaunching TUI...");
    let state = AppState::new(all_packages, &config.exclude.permanent);

    match terminal::run_tui(state) {
        Ok(Some(event)) => match event {
            UIEvent::UpdateEntireSystem => {
                println!("\nPreparing entire system update...");
                // TODO: Execute paru command
            }
            UIEvent::UpdateOfficialOnly => {
                println!("\nPreparing official-only update...");
                // TODO: Execute pacman command
            }
            UIEvent::Quit => {
                println!("\nExiting without updates.");
            }
            UIEvent::OpenLink(_) => {
                println!("\nLink opening not yet implemented.");
            }
        },
        Ok(None) => {
            println!("\nNo action taken.");
        }
        Err(e) => {
            eprintln!("\nTUI error: {}", e);
        }
    }
}
