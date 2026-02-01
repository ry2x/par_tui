use crate::models::package::Package;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct PackageItem {
    pub package: Package,
    pub is_temporarily_ignored: bool,
    pub is_permanently_ignored: bool,
}

/// Creates `PackageItem` list from packages and permanent exclusions.
///
/// Uses `HashSet` for `O(1)` exclusion lookups.
#[must_use]
pub fn create_package_items(
    packages: Vec<Package>,
    permanent_excludes: &[String],
) -> Vec<PackageItem> {
    let excludes: HashSet<&String> = permanent_excludes.iter().collect();
    packages
        .into_iter()
        .map(|pkg| {
            let is_perm = excludes.contains(&pkg.name);
            PackageItem {
                package: pkg,
                is_temporarily_ignored: false,
                is_permanently_ignored: is_perm,
            }
        })
        .collect()
}

/// Returns a list of all ignored package names (temporary + permanent).
#[must_use]
pub fn build_ignore_list(items: &[PackageItem]) -> Vec<String> {
    items
        .iter()
        .filter(|item| item.is_temporarily_ignored || item.is_permanently_ignored)
        .map(|item| item.package.name.clone())
        .collect()
}

/// Returns a list of permanently ignored package names.
#[must_use]
pub fn extract_permanent(items: &[PackageItem]) -> Vec<String> {
    items
        .iter()
        .filter(|item| item.is_permanently_ignored)
        .map(|item| item.package.name.clone())
        .collect()
}

/// Returns package statistics: (`official_count`, `aur_count`, `ignored_count`).
#[must_use]
pub fn calculate_stats(items: &[PackageItem]) -> (usize, usize, usize) {
    let official = items
        .iter()
        .filter(|p| {
            matches!(
                p.package.repository,
                crate::models::package::PackageRepository::Official
            )
        })
        .count();
    let aur = items.len() - official;
    let ignored = items
        .iter()
        .filter(|p| p.is_temporarily_ignored || p.is_permanently_ignored)
        .count();
    (official, aur, ignored)
}
