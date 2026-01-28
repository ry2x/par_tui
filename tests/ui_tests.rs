use par_tui::models::package::{Package, PackageRepository};
use par_tui::ui::app::AppState;

fn make_test_package(name: &str, repo: PackageRepository) -> Package {
    Package {
        name: name.to_string(),
        current_version: Some("1.0.0".to_string()),
        new_version: "2.0.0".to_string(),
        repository: repo,
    }
}

#[test]
fn test_app_state_creation() {
    let packages = vec![
        make_test_package("pkg1", PackageRepository::Official),
        make_test_package("pkg2", PackageRepository::Aur),
    ];
    let permanent = vec!["pkg2".to_string()];

    let state = AppState::new(packages, &permanent);

    assert_eq!(state.packages.len(), 2);
    assert_eq!(state.cursor_position, 0);
    assert!(!state.show_help);
    assert!(state.packages[1].is_permanently_ignored);
    assert!(!state.packages[0].is_permanently_ignored);
}

#[test]
fn test_cursor_movement() {
    let packages = vec![
        make_test_package("pkg1", PackageRepository::Official),
        make_test_package("pkg2", PackageRepository::Official),
        make_test_package("pkg3", PackageRepository::Official),
    ];

    let mut state = AppState::new(packages, &[]);

    assert_eq!(state.cursor_position, 0);

    state.move_cursor_down();
    assert_eq!(state.cursor_position, 1);

    state.move_cursor_down();
    assert_eq!(state.cursor_position, 2);

    state.move_cursor_down(); // Should not go beyond
    assert_eq!(state.cursor_position, 2);

    state.move_cursor_up();
    assert_eq!(state.cursor_position, 1);

    state.move_cursor_up();
    assert_eq!(state.cursor_position, 0);

    state.move_cursor_up(); // Should not go below 0
    assert_eq!(state.cursor_position, 0);
}

#[test]
fn test_toggle_package() {
    let packages = vec![
        make_test_package("pkg1", PackageRepository::Official),
        make_test_package("pkg2", PackageRepository::Official),
    ];

    let mut state = AppState::new(packages, &[]);

    assert!(!state.packages[0].is_temporarily_ignored);

    state.toggle_current_package();
    assert!(state.packages[0].is_temporarily_ignored);

    state.toggle_current_package();
    assert!(!state.packages[0].is_temporarily_ignored);
}

#[test]
fn test_cannot_toggle_permanent() {
    let packages = vec![make_test_package("pkg1", PackageRepository::Official)];
    let permanent = vec!["pkg1".to_string()];

    let mut state = AppState::new(packages, &permanent);

    assert!(state.packages[0].is_permanently_ignored);
    assert!(!state.packages[0].is_temporarily_ignored);

    state.toggle_current_package();

    // Should not change temporary flag for permanently ignored
    assert!(!state.packages[0].is_temporarily_ignored);
}

#[test]
fn test_get_ignored_packages() {
    let packages = vec![
        make_test_package("pkg1", PackageRepository::Official),
        make_test_package("pkg2", PackageRepository::Official),
        make_test_package("pkg3", PackageRepository::Official),
    ];
    let permanent = vec!["pkg2".to_string()];

    let mut state = AppState::new(packages, &permanent);
    state.toggle_current_package(); // Toggle pkg1

    let ignored = state.get_ignored_packages();

    assert_eq!(ignored.len(), 2);
    assert!(ignored.contains(&"pkg1".to_string()));
    assert!(ignored.contains(&"pkg2".to_string()));
}

#[test]
fn test_stats() {
    let packages = vec![
        make_test_package("pkg1", PackageRepository::Official),
        make_test_package("pkg2", PackageRepository::Official),
        make_test_package("pkg3", PackageRepository::Aur),
        make_test_package("pkg4", PackageRepository::Aur),
    ];
    let permanent = vec!["pkg2".to_string()];

    let mut state = AppState::new(packages, &permanent);
    state.toggle_current_package(); // Toggle pkg1

    let (official, aur, ignored) = state.stats();

    assert_eq!(official, 2);
    assert_eq!(aur, 2);
    assert_eq!(ignored, 2); // pkg1 (temp) + pkg2 (perm)
}

#[test]
fn test_toggle_help() {
    let packages = vec![make_test_package("pkg1", PackageRepository::Official)];
    let mut state = AppState::new(packages, &[]);

    assert!(!state.show_help);

    state.toggle_help();
    assert!(state.show_help);

    state.toggle_help();
    assert!(!state.show_help);
}

#[test]
fn test_toggle_permanent_ignore() {
    let packages = vec![
        make_test_package("pkg1", PackageRepository::Official),
        make_test_package("pkg2", PackageRepository::Official),
    ];
    let permanent = vec!["pkg2".to_string()];

    let mut state = AppState::new(packages, &permanent);

    // pkg1 starts as not permanently ignored
    assert!(!state.packages[0].is_permanently_ignored);

    // Toggle permanent ignore on pkg1
    state.toggle_permanent_ignore();
    assert!(state.packages[0].is_permanently_ignored);

    // Toggle again to remove permanent ignore
    state.toggle_permanent_ignore();
    assert!(!state.packages[0].is_permanently_ignored);
}

#[test]
fn test_toggle_permanent_clears_temporary() {
    let packages = vec![make_test_package("pkg1", PackageRepository::Official)];
    let mut state = AppState::new(packages, &[]);

    // Set temporary ignore first
    state.toggle_current_package();
    assert!(state.packages[0].is_temporarily_ignored);
    assert!(!state.packages[0].is_permanently_ignored);

    // Toggle permanent ignore should clear temporary
    state.toggle_permanent_ignore();
    assert!(state.packages[0].is_permanently_ignored);
    assert!(!state.packages[0].is_temporarily_ignored);
}

#[test]
fn test_get_permanent_excludes() {
    let packages = vec![
        make_test_package("pkg1", PackageRepository::Official),
        make_test_package("pkg2", PackageRepository::Official),
        make_test_package("pkg3", PackageRepository::Official),
    ];
    let permanent = vec!["pkg2".to_string()];

    let mut state = AppState::new(packages, &permanent);

    // Toggle pkg1 to permanent ignore
    state.toggle_permanent_ignore();

    let permanent_list = state.get_permanent_excludes();

    assert_eq!(permanent_list.len(), 2);
    assert!(permanent_list.contains(&"pkg1".to_string()));
    assert!(permanent_list.contains(&"pkg2".to_string()));
}
