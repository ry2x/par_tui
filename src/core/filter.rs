use crate::models::config::Config;
use crate::models::package::Package;

/// Applies permanent package exclusions from configuration.
///
/// Returns a new vector with excluded packages filtered out.
#[must_use]
#[allow(dead_code)]
pub fn apply_permanent_excludes(packages: Vec<Package>, config: &Config) -> Vec<Package> {
    let excludes = &config.exclude.permanent;
    packages
        .into_iter()
        .filter(|pkg| !excludes.contains(&pkg.name))
        .collect()
}

/// Applies temporary package exclusions for the current run.
///
/// Returns a new vector with excluded packages filtered out.
#[must_use]
#[allow(dead_code)]
pub fn apply_temporary_excludes(packages: Vec<Package>, temp_excludes: &[String]) -> Vec<Package> {
    packages
        .into_iter()
        .filter(|pkg| !temp_excludes.contains(&pkg.name))
        .collect()
}
