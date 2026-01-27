mod core;
mod io;
mod models;
mod parser;
mod ui;

use io::{command, file};
use parser::{pacman, paru, toml as toml_parser};
use core::filter;
use std::path::PathBuf;

fn main() {
    println!("par_tui - Arch Linux Update Manager\n");

    // Load config
    let config_home = std::env::var("XDG_CONFIG_HOME")
        .unwrap_or_else(|_| {
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
                println!("  Permanent excludes: {:?}", cfg.exclude.permanent);
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
    println!("\nChecking for required commands...");
    let has_checkupdates = command::check_command_exists("checkupdates");
    let has_paru = command::check_command_exists("paru");

    println!("  checkupdates: {}", if has_checkupdates { "✓" } else { "✗" });
    println!("  paru: {}", if has_paru { "✓" } else { "✗" });

    if !has_checkupdates {
        eprintln!("\nError: checkupdates not found. Install pacman-contrib.");
        return;
    }

    // Scan for official updates
    println!("\nScanning for official package updates...");
    let mut all_packages = Vec::new();
    match command::run_checkupdates() {
        Ok(output) => {
            let packages = pacman::parse_checkupdates_output(&output);
            println!("  Found {} official updates", packages.len());
            all_packages.extend(packages);
        }
        Err(e) => {
            eprintln!("  Error: {:?}", e);
        }
    }

    // Scan for AUR updates
    if has_paru {
        println!("\nScanning for AUR package updates...");
        match command::run_paru_query_aur() {
            Ok(output) => {
                let packages = paru::parse_paru_output(&output);
                println!("  Found {} AUR updates", packages.len());
                all_packages.extend(packages);
            }
            Err(e) => {
                eprintln!("  Error: {:?}", e);
            }
        }
    }

    // Apply filters
    let total_before = all_packages.len();
    let filtered = filter::apply_permanent_excludes(all_packages, &config);
    let total_after = filtered.len();

    println!("\nAfter applying permanent excludes:");
    println!("  {} packages remain ({} excluded)", total_after, total_before - total_after);
    
    for pkg in &filtered {
        println!("    - {}: {} → {}", pkg.name,
            pkg.current_version.as_ref().unwrap_or(&"?".to_string()),
            pkg.new_version);
    }

    println!("\nScan complete.");
}
