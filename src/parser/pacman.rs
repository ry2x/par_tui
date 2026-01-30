use crate::models::package::{Package, PackageRepository};

/// Parses `checkupdates` command output into a list of packages.
///
/// Expected format: `package_name current_version -> new_version`
#[must_use]
pub fn parse_checkupdates_output(output: &str) -> Vec<Package> {
    output
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                Some(Package {
                    name: parts[0].to_string(),
                    current_version: Some(parts[1].to_string()),
                    new_version: parts[3].to_string(),
                    repository: PackageRepository::Official,
                })
            } else {
                None
            }
        })
        .collect()
}

/// Parses `pacman -Qi` output to extract the "Required By" field.
///
/// Expected format:
/// ```text
/// Required By     : pkg1  pkg2  pkg3
/// ```
///
/// Returns a vector of package names that depend on the queried package.
/// Returns empty vector if "Required By" field is "None" or not found.
#[must_use]
#[allow(dead_code)]
pub fn parse_required_by(output: &str) -> Vec<String> {
    for line in output.lines() {
        let trimmed = line.trim();
        if let Some(value) = trimmed.strip_prefix("Required By") {
            let value = value.trim().trim_start_matches(':').trim();
            if value == "None" || value.is_empty() {
                return Vec::new();
            }
            return value.split_whitespace().map(String::from).collect();
        }
    }
    Vec::new()
}
