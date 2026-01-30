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

#[test]
fn test_has_official_scan_failed_no_warnings() {
    let packages = vec![make_test_package("pkg1", PackageRepository::Official)];
    let state = AppState::new(packages, &[]);

    assert!(!state.has_official_scan_failed());
}

#[test]
fn test_has_official_scan_failed_with_official_failure() {
    let packages = vec![make_test_package("pkg1", PackageRepository::Official)];
    let mut state = AppState::new(packages, &[]);
    state.scan_warnings.push("Official scan failed".to_string());

    assert!(state.has_official_scan_failed());
}

#[test]
fn test_has_official_scan_failed_with_aur_failure_only() {
    let packages = vec![make_test_package("pkg1", PackageRepository::Official)];
    let mut state = AppState::new(packages, &[]);
    state.scan_warnings.push("AUR scan failed".to_string());

    assert!(!state.has_official_scan_failed());
}

#[test]
fn test_has_official_scan_failed_with_combined_failure() {
    let packages = vec![make_test_package("pkg1", PackageRepository::Official)];
    let mut state = AppState::new(packages, &[]);
    state
        .scan_warnings
        .push("Official & AUR scan failed".to_string());

    // Should detect "Official" marker even in combined message
    assert!(state.has_official_scan_failed());
}

#[test]
fn test_scan_failure_marker_constants() {
    use par_tui::io::terminal::{AUR_SCAN_FAILURE_MARKER, OFFICIAL_SCAN_FAILURE_MARKER};

    // Verify constants are what we expect
    assert_eq!(OFFICIAL_SCAN_FAILURE_MARKER, "Official");
    assert_eq!(AUR_SCAN_FAILURE_MARKER, "AUR");
}

// Phase 2: UI Integration Tests for dependency warnings

#[test]
fn test_set_dependency_conflicts_shows_modal() {
    use par_tui::core::dependency::DependencyConflict;

    let packages = vec![make_test_package("pkg1", PackageRepository::Official)];
    let mut state = AppState::new(packages, &[]);

    assert!(!state.show_dependency_warning);

    let conflicts = vec![DependencyConflict {
        ignored_package: "glibc".to_string(),
        required_by: vec!["systemd".to_string()],
    }];

    state.set_dependency_conflicts(conflicts);

    assert!(state.show_dependency_warning);
    assert_eq!(state.dependency_conflicts.len(), 1);
}

#[test]
fn test_toggle_dependency_warning() {
    let packages = vec![make_test_package("pkg1", PackageRepository::Official)];
    let mut state = AppState::new(packages, &[]);

    assert!(!state.show_dependency_warning);

    state.toggle_dependency_warning();
    assert!(state.show_dependency_warning);

    state.toggle_dependency_warning();
    assert!(!state.show_dependency_warning);
}

#[test]
fn test_has_conflicts() {
    use par_tui::core::dependency::DependencyConflict;

    let packages = vec![make_test_package("pkg1", PackageRepository::Official)];
    let mut state = AppState::new(packages, &[]);

    assert!(!state.has_conflicts());

    state.dependency_conflicts.push(DependencyConflict {
        ignored_package: "test".to_string(),
        required_by: vec!["dep".to_string()],
    });

    assert!(state.has_conflicts());
}

#[test]
fn test_pending_action_lifecycle() {
    use par_tui::ui::app::UIEvent;

    let packages = vec![make_test_package("pkg1", PackageRepository::Official)];
    let mut state = AppState::new(packages, &[]);

    // Initially None
    assert!(state.pending_action.is_none());

    // Set action
    state.pending_action = Some(UIEvent::UpdateEntireSystem);
    assert!(state.pending_action.is_some());

    // Take consumes it
    let action = state.pending_action.take();
    assert!(matches!(action, Some(UIEvent::UpdateEntireSystem)));
    assert!(state.pending_action.is_none());

    // Clear sets to None
    state.pending_action = Some(UIEvent::UpdateOfficialOnly);
    state.pending_action = None;
    assert!(state.pending_action.is_none());
}

#[test]
fn test_reverse_deps_cache() {
    let packages = vec![make_test_package("pkg1", PackageRepository::Official)];
    let mut state = AppState::new(packages, &[]);

    let mut call_count = 0;

    // First call: cache miss, fetch called
    let (deps1, err1) = state.get_or_fetch_required_by("glibc", || {
        call_count += 1;
        Ok(vec!["systemd".to_string(), "bash".to_string()])
    });
    assert_eq!(call_count, 1);
    assert_eq!(deps1.len(), 2);
    assert!(err1.is_none());

    // Second call: cache hit, fetch not called
    let (deps2, err2) = state.get_or_fetch_required_by("glibc", || {
        call_count += 1;
        Ok(vec![]) // Should not be reached
    });
    assert_eq!(call_count, 1); // No increment - cache was used
    assert_eq!(deps2.len(), 2);
    assert!(err2.is_none());

    // Different package: cache miss again
    let (deps3, err3) = state.get_or_fetch_required_by("bash", || {
        call_count += 1;
        Ok(vec!["base".to_string()])
    });
    assert_eq!(call_count, 2);
    assert_eq!(deps3.len(), 1);
    assert!(err3.is_none());
}

#[test]
fn test_reverse_deps_cache_error_handling() {
    let packages = vec![make_test_package("pkg1", PackageRepository::Official)];
    let mut state = AppState::new(packages, &[]);

    // Fetch error: not cached
    let (deps1, err1) =
        state.get_or_fetch_required_by("nonexistent", || Err("Package not found".to_string()));
    assert!(deps1.is_empty());
    assert_eq!(err1, Some("Package not found".to_string()));

    // Error result is not cached: fetch called again
    let (deps2, err2) =
        state.get_or_fetch_required_by("nonexistent", || Err("Still not found".to_string()));
    assert!(deps2.is_empty());
    assert_eq!(err2, Some("Still not found".to_string()));
}

#[test]
fn test_is_ready_states() {
    use par_tui::ui::app::LoadingState;

    let packages = vec![make_test_package("pkg1", PackageRepository::Official)];
    let mut state = AppState::new(packages, &[]);

    // Ready state
    state.loading_state = LoadingState::Ready;
    assert!(state.is_ready());

    // NoUpdates state
    state.loading_state = LoadingState::NoUpdates;
    assert!(state.is_ready());

    // Scanning state (not ready)
    state.loading_state = LoadingState::Scanning;
    assert!(!state.is_ready());

    // Error state (not ready)
    state.loading_state = LoadingState::Error("test".to_string());
    assert!(!state.is_ready());
}
