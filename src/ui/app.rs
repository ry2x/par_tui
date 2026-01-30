use crate::models::package::Package;
use crate::core::dependency::DependencyConflict;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum UIEvent {
    UpdateEntireSystem,
    UpdateOfficialOnly,
    Reload,
    Quit,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum LoadingState {
    Scanning,
    Ready,
    NoUpdates,
    Error(String),
}

pub struct AppState {
    pub packages: Vec<PackageItem>,
    pub cursor_position: usize,
    pub show_help: bool,
    pub loading_state: LoadingState,
    pub loading_message: String,
    pub scan_warnings: Vec<String>,
    pub dependency_conflicts: Vec<DependencyConflict>,
    pub show_dependency_warning: bool,
    
    /// Pending action lifecycle:
    /// 1. Set when Enter/o pressed (before dependency check)
    /// 2. Held during dependency warning modal display
    /// 3. Consumed (`take()`) when user confirms (y)
    /// 4. Cleared (None) when user cancels (n/Esc)
    pub pending_action: Option<UIEvent>,
    
    /// Cache for `pacman -Qi` reverse dependency queries
    /// Key: package name, Value: list of packages requiring it
    pub reverse_deps_cache: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct PackageItem {
    pub package: Package,
    pub is_temporarily_ignored: bool,
    pub is_permanently_ignored: bool,
}

impl AppState {
    /// Creates a new `AppState` in loading state.
    #[must_use]
    pub fn new_loading() -> Self {
        Self {
            packages: Vec::new(),
            cursor_position: 0,
            show_help: false,
            loading_state: LoadingState::Scanning,
            loading_message: "Initializing...".to_string(),
            scan_warnings: Vec::new(),
            dependency_conflicts: Vec::new(),
            show_dependency_warning: false,
            pending_action: None,
            reverse_deps_cache: HashMap::new(),
        }
    }

    /// Creates a new `AppState` with the given packages and permanent exclusions.
    #[must_use]
    #[allow(dead_code)]
    pub fn new(packages: Vec<Package>, permanent_excludes: &[String]) -> Self {
        let items = Self::create_package_items(packages, permanent_excludes);

        Self {
            packages: items,
            cursor_position: 0,
            show_help: false,
            loading_state: LoadingState::Ready,
            loading_message: String::new(),
            scan_warnings: Vec::new(),
            dependency_conflicts: Vec::new(),
            show_dependency_warning: false,
            pending_action: None,
            reverse_deps_cache: HashMap::new(),
        }
    }

    pub fn set_loading_message<S: Into<String>>(&mut self, message: S) {
        self.loading_message = message.into();
    }

    pub fn add_scan_warning<S: Into<String>>(&mut self, warning: S) {
        self.scan_warnings.push(warning.into());
    }

    pub fn set_packages(&mut self, packages: Vec<Package>, permanent_excludes: &[String]) {
        self.packages = Self::create_package_items(packages, permanent_excludes);
        self.loading_state = LoadingState::Ready;
    }

    /// Helper to create `PackageItem` list from packages and permanent exclusions
    fn create_package_items(
        packages: Vec<Package>,
        permanent_excludes: &[String],
    ) -> Vec<PackageItem> {
        packages
            .into_iter()
            .map(|pkg| {
                let is_perm = permanent_excludes.contains(&pkg.name);
                PackageItem {
                    package: pkg,
                    is_temporarily_ignored: false,
                    is_permanently_ignored: is_perm,
                }
            })
            .collect()
    }

    pub fn set_no_updates(&mut self) {
        self.loading_state = LoadingState::NoUpdates;
    }

    #[allow(dead_code)]
    pub fn set_error<S: Into<String>>(&mut self, error: S) {
        self.loading_state = LoadingState::Error(error.into());
    }

    pub fn move_cursor_up(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    pub fn move_cursor_down(&mut self) {
        if self.cursor_position < self.packages.len().saturating_sub(1) {
            self.cursor_position += 1;
        }
    }

    pub fn toggle_current_package(&mut self) {
        if let Some(item) = self.packages.get_mut(self.cursor_position)
            && !item.is_permanently_ignored
        {
            item.is_temporarily_ignored = !item.is_temporarily_ignored;
        }
    }

    pub fn toggle_permanent_ignore(&mut self) {
        if let Some(item) = self.packages.get_mut(self.cursor_position) {
            item.is_permanently_ignored = !item.is_permanently_ignored;
            if item.is_permanently_ignored {
                item.is_temporarily_ignored = false;
            }
        }
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    /// Returns a list of all ignored package names (temporary + permanent).
    #[must_use]
    pub fn get_ignored_packages(&self) -> Vec<String> {
        self.packages
            .iter()
            .filter(|item| item.is_temporarily_ignored || item.is_permanently_ignored)
            .map(|item| item.package.name.clone())
            .collect()
    }

    /// Returns a list of permanently ignored package names.
    #[must_use]
    pub fn get_permanent_excludes(&self) -> Vec<String> {
        self.packages
            .iter()
            .filter(|item| item.is_permanently_ignored)
            .map(|item| item.package.name.clone())
            .collect()
    }

    /// Returns package statistics: (`official_count`, `aur_count`, `ignored_count`).
    #[must_use]
    pub fn stats(&self) -> (usize, usize, usize) {
        let official = self
            .packages
            .iter()
            .filter(|p| {
                matches!(
                    p.package.repository,
                    crate::models::package::PackageRepository::Official
                )
            })
            .count();
        let aur = self.packages.len() - official;
        let ignored = self
            .packages
            .iter()
            .filter(|p| p.is_temporarily_ignored || p.is_permanently_ignored)
            .count();
        (official, aur, ignored)
    }

    /// Returns true if official scan has failed
    #[must_use]
    pub fn has_official_scan_failed(&self) -> bool {
        self.scan_warnings
            .iter()
            .any(|w| w.contains(crate::io::terminal::OFFICIAL_SCAN_FAILURE_MARKER))
    }

    /// Toggles dependency warning modal visibility
    pub fn toggle_dependency_warning(&mut self) {
        self.show_dependency_warning = !self.show_dependency_warning;
    }

    /// Sets dependency conflicts and shows warning modal
    pub fn set_dependency_conflicts(&mut self, conflicts: Vec<DependencyConflict>) {
        self.dependency_conflicts = conflicts;
        if !self.dependency_conflicts.is_empty() {
            self.show_dependency_warning = true;
        }
    }

    /// Checks if there are any dependency conflicts
    #[must_use]
    #[allow(dead_code)]
    pub fn has_conflicts(&self) -> bool {
        !self.dependency_conflicts.is_empty()
    }

    /// Gets or fetches reverse dependencies for a package (with caching)
    ///
    /// Returns (dependencies, `optional_error_message`)
    pub fn get_or_fetch_required_by<F>(
        &mut self,
        pkg: &str,
        fetch: F,
    ) -> (Vec<String>, Option<String>)
    where
        F: FnOnce() -> Result<Vec<String>, String>,
    {
        if let Some(cached) = self.reverse_deps_cache.get(pkg) {
            return (cached.clone(), None);
        }

        match fetch() {
            Ok(deps) => {
                self.reverse_deps_cache.insert(pkg.to_string(), deps.clone());
                (deps, None)
            },
            Err(e) => (Vec::new(), Some(e)),
        }
    }
}
