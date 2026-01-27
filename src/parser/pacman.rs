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
