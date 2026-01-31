use crate::models::package::Package;
use std::collections::HashSet;

/// Represents a dependency conflict warning.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyConflict {
    pub ignored_package: String,
    pub required_by: Vec<String>,
}

/// Result type for dependency operations
pub type DependencyResult = Result<Vec<DependencyConflict>, Vec<String>>;

/// Checks for dependency conflicts given packages and a resolver function.
///
/// # Arguments
///
/// * `all_packages` - All packages available for update
/// * `ignored_packages` - Package names being ignored
/// * `get_required_by` - Function returning (`package_deps`, `optional_error_message`)
///
/// # Returns
///
/// Returns `Ok(conflicts)` if check succeeds, `Err(warnings)` if any package queries fail.
///
/// # Errors
///
/// Returns `Err(Vec<String>)` containing warning messages if dependency queries fail.
pub fn check_conflicts<F>(
    all_packages: &[Package],
    ignored_packages: &[String],
    mut get_required_by: F,
) -> DependencyResult
where
    F: FnMut(&str) -> (Vec<String>, Option<String>),
{
    let mut warnings = Vec::new();

    let conflicts = detect_conflicts(all_packages, ignored_packages, |pkg| {
        let (deps, err) = get_required_by(pkg);
        if let Some(e) = err {
            warnings.push(format!("Failed to check dependencies for {pkg}: {e}"));
        }
        deps
    });

    if warnings.is_empty() {
        Ok(conflicts)
    } else {
        Err(warnings)
    }
}

/// Detects dependency conflicts when packages are ignored.
///
/// Checks if any packages being updated depend on packages being ignored.
/// This prevents partial upgrade scenarios that could break the system.
///
/// # Arguments
///
/// * `all_packages` - All packages available for update
/// * `ignored_packages` - Package names being ignored (temporary or permanent)
/// * `get_required_by` - Function to fetch reverse dependencies for a package
///
/// # Returns
///
/// Vector of conflicts, where ignored packages are required by packages being updated.
pub fn detect_conflicts<F>(
    all_packages: &[Package],
    ignored_packages: &[String],
    mut get_required_by: F,
) -> Vec<DependencyConflict>
where
    F: FnMut(&str) -> Vec<String>,
{
    let ignored_set: HashSet<&str> = ignored_packages.iter().map(String::as_str).collect();
    let updating_packages: HashSet<&str> = all_packages
        .iter()
        .map(|p| p.name.as_str())
        .filter(|name| !ignored_set.contains(name))
        .collect();

    let mut conflicts = Vec::new();

    for ignored in ignored_packages {
        let required_by = get_required_by(ignored);

        // Find conflicts: packages that are being updated and require this ignored package
        let mut conflicting: Vec<String> = required_by
            .iter()
            .filter(|dep| updating_packages.contains(dep.as_str()))
            .cloned()
            .collect();

        if !conflicting.is_empty() {
            conflicting.sort();
            conflicts.push(DependencyConflict {
                ignored_package: ignored.clone(),
                required_by: conflicting,
            });
        }
    }

    conflicts.sort_by(|a, b| a.ignored_package.cmp(&b.ignored_package));
    conflicts
}
