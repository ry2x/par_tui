#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Package {
    pub name: String,
    pub current_version: Option<String>,
    pub new_version: String,
    pub repository: PackageRepository,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageRepository {
    Official,
    Aur,
}
