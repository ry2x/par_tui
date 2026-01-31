use par_tui::core::dependency::{check_conflicts, detect_conflicts};
use par_tui::models::package::{Package, PackageRepository};

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

#[test]
fn test_check_conflicts_with_warnings() {
    let packages = vec![make_package("pkg1"), make_package("pkg2")];
    let ignored = vec!["glibc".to_string(), "base".to_string()];

    let get_required_by = |name: &str| -> (Vec<String>, Option<String>) {
        match name {
            "glibc" => (vec![], Some("Failed to query glibc".to_string())),
            "base" => (vec!["pkg1".to_string()], None),
            _ => (vec![], None),
        }
    };

    let result = check_conflicts(&packages, &ignored, get_required_by);

    assert!(result.is_err());
    if let Err(warnings) = result {
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("Failed to check dependencies for glibc"));
    }
}

#[test]
fn test_check_conflicts_success() {
    let packages = vec![make_package("pkg1"), make_package("pkg2")];
    let ignored = vec!["glibc".to_string()];

    let get_required_by = |name: &str| -> (Vec<String>, Option<String>) {
        match name {
            "glibc" => (vec!["pkg1".to_string(), "pkg2".to_string()], None),
            _ => (vec![], None),
        }
    };

    let result = check_conflicts(&packages, &ignored, get_required_by);

    assert!(result.is_ok());
    if let Ok(conflicts) = result {
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].ignored_package, "glibc");
        assert_eq!(conflicts[0].required_by.len(), 2);
    }
}

#[test]
fn test_check_conflicts_no_conflicts_no_warnings() {
    let packages = vec![make_package("pkg1"), make_package("pkg2")];
    let ignored = vec!["unrelated".to_string()];

    let get_required_by = |_: &str| -> (Vec<String>, Option<String>) { (vec![], None) };

    let result = check_conflicts(&packages, &ignored, get_required_by);

    assert!(result.is_ok());
    if let Ok(conflicts) = result {
        assert!(conflicts.is_empty());
    }
}
