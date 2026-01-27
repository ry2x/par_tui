use crate::models::package::Package;

#[derive(Debug, Clone)]
pub enum UIEvent {
    UpdateEntireSystem,
    UpdateOfficialOnly,
    OpenLink(String),
    Quit,
}

pub struct AppState {
    pub packages: Vec<PackageItem>,
    pub cursor_position: usize,
    pub show_help: bool,
}

#[derive(Debug, Clone)]
pub struct PackageItem {
    pub package: Package,
    pub is_temporarily_ignored: bool,
    pub is_permanently_ignored: bool,
}

impl AppState {
    /// Creates a new `AppState` with the given packages and permanent exclusions.
    #[must_use]
    pub fn new(packages: Vec<Package>, permanent_excludes: &[String]) -> Self {
        let items = packages
            .into_iter()
            .map(|pkg| {
                let is_perm = permanent_excludes.contains(&pkg.name);
                PackageItem {
                    package: pkg,
                    is_temporarily_ignored: false,
                    is_permanently_ignored: is_perm,
                }
            })
            .collect();

        Self {
            packages: items,
            cursor_position: 0,
            show_help: false,
        }
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
            && !item.is_permanently_ignored {
                item.is_temporarily_ignored = !item.is_temporarily_ignored;
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

    /// Returns package statistics: (`official_count`, `aur_count`, `ignored_count`).
    #[must_use]
    pub fn stats(&self) -> (usize, usize, usize) {
        let official = self
            .packages
            .iter()
            .filter(|p| matches!(p.package.repository, crate::models::package::PackageRepository::Official))
            .count();
        let aur = self.packages.len() - official;
        let ignored = self
            .packages
            .iter()
            .filter(|p| p.is_temporarily_ignored || p.is_permanently_ignored)
            .count();
        (official, aur, ignored)
    }
}
