use par_tui::models::package::{Package, PackageRepository};
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
    let output = r#"linux 6.1.10-1 -> 6.1.12-1
mesa 23.0.1-1 -> 23.0.2-1
systemd 253.1-1 -> 253.2-1
"#;
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
    let output = r#"linux 6.1.10-1 -> 6.1.12-1
invalid line
mesa 23.0.1-1 -> 23.0.2-1
"#;
    let packages = pacman::parse_checkupdates_output(output);

    // Should skip malformed line
    assert_eq!(packages.len(), 2);
    assert_eq!(packages[0].name, "linux");
    assert_eq!(packages[1].name, "mesa");
}
