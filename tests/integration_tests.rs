use par_tui::core::planner::{UpdateMode, create_plan};
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
fn test_entire_system_command_dry_run() {
    let packages = vec![
        make_test_package("pkg1", PackageRepository::Official),
        make_test_package("pkg2", PackageRepository::Aur),
    ];
    let ignored = vec!["pkg2".to_string()];

    let plan = create_plan(UpdateMode::EntireSystem, packages, ignored);
    let config = Config::default();
    let cmd = plan.build_command(&config);

    assert_eq!(cmd.join(" "), "paru -Syu --ignore pkg2");
}

#[test]
fn test_official_only_command_dry_run() {
    let packages = vec![
        make_test_package("pkg1", PackageRepository::Official),
        make_test_package("pkg2", PackageRepository::Aur),
    ];
    let ignored = vec!["pkg1".to_string()];

    let plan = create_plan(UpdateMode::OfficialOnly, packages, ignored);
    let config = Config::default();
    let cmd = plan.build_command(&config);

    assert_eq!(cmd.join(" "), "sudo pacman -Syu --ignore pkg1");
}

#[test]
fn test_no_ignore_entire_system() {
    let packages = vec![
        make_test_package("pkg1", PackageRepository::Official),
        make_test_package("pkg2", PackageRepository::Aur),
    ];

    let plan = create_plan(UpdateMode::EntireSystem, packages, vec![]);
    let config = Config::default();
    let cmd = plan.build_command(&config);

    assert_eq!(cmd.join(" "), "paru -Syu");
    assert!(!cmd.contains(&"--ignore".to_string()));
}

#[test]
fn test_no_ignore_official_only() {
    let packages = vec![make_test_package("pkg1", PackageRepository::Official)];

    let plan = create_plan(UpdateMode::OfficialOnly, packages, vec![]);
    let config = Config::default();
    let cmd = plan.build_command(&config);

    assert_eq!(cmd.join(" "), "sudo pacman -Syu");
    assert!(!cmd.contains(&"--ignore".to_string()));
}

#[test]
fn test_multiple_ignores() {
    let packages = vec![
        make_test_package("pkg1", PackageRepository::Official),
        make_test_package("pkg2", PackageRepository::Official),
    ];
    let ignored = vec!["pkg1".to_string(), "pkg2".to_string(), "pkg3".to_string()];

    let plan = create_plan(UpdateMode::EntireSystem, packages, ignored);
    let config = Config::default();
    let cmd = plan.build_command(&config);

    assert_eq!(cmd.join(" "), "paru -Syu --ignore pkg1,pkg2,pkg3");
}

#[test]
fn test_extra_args_in_command() {
    let packages = vec![make_test_package("pkg1", PackageRepository::Official)];

    let plan = create_plan(UpdateMode::EntireSystem, packages, vec![]);
    let mut config = Config::default();
    config.behavior.extra_args = vec!["--noconfirm".to_string(), "--needed".to_string()];
    let cmd = plan.build_command(&config);

    assert_eq!(cmd.join(" "), "paru -Syu --noconfirm --needed");
}
