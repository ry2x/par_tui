use crossterm::event::KeyCode;

use super::app::{AppState, LoadingState, UIEvent};

/// Result from handling key events in modals
pub enum ModalResult {
    Proceed(Option<UIEvent>),
    Cancel,
    Quit,
    IgnoreKey,
}

/// Handles key events in the main application state.
///
/// Returns `Some(UIEvent)` for terminating events, `None` to continue loop.
pub fn handle_key_event(state: &mut AppState, key_code: KeyCode) -> Option<UIEvent> {
    // Handle dependency warning modal first (blocks all other input)
    if state.show_dependency_warning {
        match handle_dependency_warning_modal(state, key_code) {
            ModalResult::Proceed(event) => return event,
            ModalResult::Quit => return Some(UIEvent::Quit),
            ModalResult::Cancel | ModalResult::IgnoreKey => return None,
        }
    }

    match (&state.loading_state, key_code) {
        // Allow quit in any state
        (_, KeyCode::Char('q')) => Some(UIEvent::Quit),

        // Allow reload if official scan failed
        (LoadingState::Ready | LoadingState::NoUpdates, KeyCode::Char('r'))
            if state.has_official_scan_failed() =>
        {
            Some(UIEvent::Reload)
        }

        // Only allow other keys when ready
        (LoadingState::Ready, KeyCode::Char('?')) => {
            state.toggle_help();
            None
        }
        (LoadingState::Ready, KeyCode::Char('j') | KeyCode::Down) => {
            state.move_cursor_down();
            None
        }
        (LoadingState::Ready, KeyCode::Char('k') | KeyCode::Up) => {
            state.move_cursor_up();
            None
        }
        (LoadingState::Ready, KeyCode::Char('p')) => {
            state.toggle_permanent_ignore();
            None
        }
        (LoadingState::Ready, KeyCode::Char(' ')) => {
            state.toggle_current_package();
            None
        }
        (LoadingState::Ready, KeyCode::Char('o')) => {
            state.pending_action = Some(UIEvent::UpdateOfficialOnly);
            Some(UIEvent::UpdateOfficialOnly)
        }
        (LoadingState::Ready, KeyCode::Enter) => {
            state.pending_action = Some(UIEvent::UpdateEntireSystem);
            Some(UIEvent::UpdateEntireSystem)
        }
        _ => None,
    }
}

/// Handles key events in dependency warning modal.
pub fn handle_dependency_warning_modal(state: &mut AppState, key_code: KeyCode) -> ModalResult {
    match key_code {
        KeyCode::Char('y') => {
            state.toggle_dependency_warning();
            ModalResult::Proceed(state.pending_action.take())
        }
        KeyCode::Char('n') | KeyCode::Esc => {
            state.toggle_dependency_warning();
            state.pending_action = None;
            ModalResult::Cancel
        }
        KeyCode::Char('q') => {
            state.pending_action = None;
            ModalResult::Quit
        }
        _ => ModalResult::IgnoreKey,
    }
}
