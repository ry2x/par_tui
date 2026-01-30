use crate::models::package::Package;
use std::collections::HashSet;

/// Represents a dependency conflict warning.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
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
#[allow(dead_code)]
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
        let conflicting: Vec<String> = required_by
            .iter()
            .filter(|dep| updating_packages.contains(dep.as_str()))
            .cloned()
            .collect();

        if !conflicting.is_empty() {
            conflicts.push(DependencyConflict {
                ignored_package: ignored.clone(),
                required_by: conflicting,
            });
        }
    }

    conflicts
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::package::PackageRepository;

    fn make_package(name: &str) -> Package {
        Package {
            name: name.to_string(),
            current_version: Some("1.0.0".to_string()),
            new_version: "2.0.0".to_string(),
            repository: PackageRepository::Official,
        }
    }

    #[test]
    fn test_no_conflicts() {
        let packages = vec![make_package("pkg1"), make_package("pkg2")];
        let ignored = vec!["pkg3".to_string()];

        let conflicts = detect_conflicts(&packages, &ignored, |_| Vec::new());

        assert!(conflicts.is_empty());
    }

    #[test]
    fn test_conflict_detected() {
        let packages = vec![
            make_package("pkg1"),
            make_package("pkg2"),
            make_package("pkg3"),
        ];
        let ignored = vec!["pkg2".to_string()];

        let get_deps = |name: &str| -> Vec<String> {
            match name {
                "pkg2" => vec!["pkg1".to_string(), "pkg3".to_string()],
                _ => Vec::new(),
            }
        };

        let conflicts = detect_conflicts(&packages, &ignored, get_deps);

        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].ignored_package, "pkg2");
        assert_eq!(conflicts[0].required_by.len(), 2);
        assert!(conflicts[0].required_by.contains(&"pkg1".to_string()));
        assert!(conflicts[0].required_by.contains(&"pkg3".to_string()));
    }

    #[test]
    fn test_ignored_package_not_required() {
        let packages = vec![make_package("pkg1"), make_package("pkg2")];
        let ignored = vec!["pkg3".to_string()];

        let get_deps = |name: &str| -> Vec<String> {
            match name {
                "pkg3" => vec!["pkg4".to_string(), "pkg5".to_string()],
                _ => Vec::new(),
            }
        };

        let conflicts = detect_conflicts(&packages, &ignored, get_deps);

        assert!(conflicts.is_empty());
    }

    #[test]
    fn test_multiple_conflicts() {
        let packages = vec![
            make_package("pkg1"),
            make_package("pkg2"),
            make_package("pkg3"),
        ];
        let ignored = vec!["base".to_string(), "glibc".to_string()];

        let get_deps = |name: &str| -> Vec<String> {
            match name {
                "base" => vec!["pkg1".to_string()],
                "glibc" => vec!["pkg2".to_string(), "pkg3".to_string()],
                _ => Vec::new(),
            }
        };

        let conflicts = detect_conflicts(&packages, &ignored, get_deps);

        assert_eq!(conflicts.len(), 2);
    }
}
