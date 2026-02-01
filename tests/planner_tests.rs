use par_tui::core::planner::{create_plan, UpdateMode};
use par_tui::models::config::Config;
use par_tui::models::package::{Package, PackageRepository};

fn make_test_package(name: &str, repo: PackageRepository) -> Package {
    Package {
        name: name.to_string(),
        current_version: Some("1.0.0".to_string()),
        new_version: "2.0.0".to_string(),
        repository: repo,
    }
}

#[test]
fn test_build_command_entire_system() {
    let packages = vec![make_test_package("pkg1", PackageRepository::Official)];
    let plan = create_plan(UpdateMode::EntireSystem, packages, vec![]);

    let config = Config::default();
    let cmd = plan.build_command(&config);

    assert_eq!(cmd[0], "paru");
    assert_eq!(cmd[1], "-Syu");
}

#[test]
fn test_build_command_official_only() {
    let packages = vec![make_test_package("pkg1", PackageRepository::Official)];
    let plan = create_plan(UpdateMode::OfficialOnly, packages, vec![]);

    let config = Config::default();
    let cmd = plan.build_command(&config);

    assert_eq!(cmd[0], "sudo");
    assert_eq!(cmd[1], "pacman");
    assert_eq!(cmd[2], "-Syu");
}

#[test]
fn test_build_command_with_ignore() {
    let packages = vec![make_test_package("pkg1", PackageRepository::Official)];
    let ignored = vec!["pkg2".to_string(), "pkg3".to_string()];
    let plan = create_plan(UpdateMode::EntireSystem, packages, ignored);

    let config = Config::default();
    let cmd = plan.build_command(&config);

    assert!(cmd.contains(&"--ignore".to_string()));
    assert!(cmd.contains(&"pkg2,pkg3".to_string()));
}

#[test]
fn test_build_command_with_extra_args() {
    let packages = vec![make_test_package("pkg1", PackageRepository::Official)];
    let plan = create_plan(UpdateMode::EntireSystem, packages, vec![]);

    let mut config = Config::default();
    config.behavior.extra_args = vec!["--noconfirm".to_string()];
    let cmd = plan.build_command(&config);

    assert!(cmd.contains(&"--noconfirm".to_string()));
}
