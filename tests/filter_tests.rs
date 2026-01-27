use par_tui::core::filter::{apply_permanent_excludes, apply_temporary_excludes};
use par_tui::models::config::Config;
use par_tui::models::package::{Package, PackageRepository};

fn make_test_package(name: &str) -> Package {
    Package {
        name: name.to_string(),
        current_version: Some("1.0.0".to_string()),
        new_version: "2.0.0".to_string(),
        repository: PackageRepository::Official,
    }
}

#[test]
fn test_apply_permanent_excludes_none() {
    let packages = vec![make_test_package("pkg1"), make_test_package("pkg2")];

    let config = Config::default();
    let filtered = apply_permanent_excludes(packages.clone(), &config);

    assert_eq!(filtered.len(), 2);
}

#[test]
fn test_apply_permanent_excludes_some() {
    let packages = vec![
        make_test_package("pkg1"),
        make_test_package("pkg2"),
        make_test_package("pkg3"),
    ];

    let mut config = Config::default();
    config.exclude.permanent = vec!["pkg2".to_string()];
    let filtered = apply_permanent_excludes(packages, &config);

    assert_eq!(filtered.len(), 2);
    assert!(filtered.iter().all(|p| p.name != "pkg2"));
}

#[test]
fn test_apply_permanent_excludes_all() {
    let packages = vec![make_test_package("pkg1"), make_test_package("pkg2")];

    let mut config = Config::default();
    config.exclude.permanent = vec!["pkg1".to_string(), "pkg2".to_string()];
    let filtered = apply_permanent_excludes(packages, &config);

    assert_eq!(filtered.len(), 0);
}

#[test]
fn test_apply_temporary_excludes() {
    let packages = vec![
        make_test_package("pkg1"),
        make_test_package("pkg2"),
        make_test_package("pkg3"),
    ];

    let temp_excludes = vec!["pkg1".to_string(), "pkg3".to_string()];
    let filtered = apply_temporary_excludes(packages, &temp_excludes);

    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].name, "pkg2");
}

#[test]
fn test_apply_temporary_excludes_empty() {
    let packages = vec![make_test_package("pkg1"), make_test_package("pkg2")];

    let filtered = apply_temporary_excludes(packages.clone(), &[]);

    assert_eq!(filtered.len(), 2);
}
