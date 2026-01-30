use par_tui::models::package::PackageRepository;
use par_tui::parser::pacman;

#[test]
fn test_parse_checkupdates_single_package() {
    let output = "linux 6.1.10-1 -> 6.1.12-1\n";
    let packages = pacman::parse_checkupdates_output(output);

    assert_eq!(packages.len(), 1);
    assert_eq!(packages[0].name, "linux");
    assert_eq!(packages[0].current_version, Some("6.1.10-1".to_string()));
    assert_eq!(packages[0].new_version, "6.1.12-1");
    assert_eq!(packages[0].repository, PackageRepository::Official);
}

#[test]
fn test_parse_checkupdates_multiple_packages() {
    let output = r"linux 6.1.10-1 -> 6.1.12-1
mesa 23.0.1-1 -> 23.0.2-1
systemd 253.1-1 -> 253.2-1
";
    let packages = pacman::parse_checkupdates_output(output);

    assert_eq!(packages.len(), 3);
    assert_eq!(packages[0].name, "linux");
    assert_eq!(packages[1].name, "mesa");
    assert_eq!(packages[2].name, "systemd");
}

#[test]
fn test_parse_checkupdates_empty() {
    let output = "";
    let packages = pacman::parse_checkupdates_output(output);

    assert_eq!(packages.len(), 0);
}

#[test]
fn test_parse_checkupdates_malformed_line() {
    let output = r"linux 6.1.10-1 -> 6.1.12-1
invalid line
mesa 23.0.1-1 -> 23.0.2-1
";
    let packages = pacman::parse_checkupdates_output(output);

    // Should skip malformed line
    assert_eq!(packages.len(), 2);
    assert_eq!(packages[0].name, "linux");
    assert_eq!(packages[1].name, "mesa");
}

#[test]
fn test_parse_required_by_none() {
    let output = r"Name            : test-package
Version         : 1.0.0-1
Required By     : None
";
    let deps = pacman::parse_required_by(output);
    assert!(deps.is_empty());
}

#[test]
fn test_parse_required_by_single() {
    let output = r"Name            : bash
Required By     : base
";
    let deps = pacman::parse_required_by(output);
    assert_eq!(deps.len(), 1);
    assert_eq!(deps[0], "base");
}

#[test]
fn test_parse_required_by_multiple() {
    let output = r"Name            : glibc
Required By     : bash  systemd  pacman
";
    let deps = pacman::parse_required_by(output);
    assert_eq!(deps.len(), 3);
    assert!(deps.contains(&"bash".to_string()));
    assert!(deps.contains(&"systemd".to_string()));
    assert!(deps.contains(&"pacman".to_string()));
}

#[test]
fn test_parse_required_by_not_found() {
    let output = r"Name            : test-package
Version         : 1.0.0-1
";
    let deps = pacman::parse_required_by(output);
    assert!(deps.is_empty());
}
