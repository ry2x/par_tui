use crossterm::{
    event::{self, Event},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

use crate::models::config::Config;
use crate::ui::{
    app::{AppState, UIEvent},
    controller,
    view,
};

/// Message types for scan thread communication (re-export from lib root)
pub use crate::ScanMessage;

/// Scan failure marker constants (re-export from lib root)
pub use crate::{AUR_SCAN_FAILURE_MARKER, OFFICIAL_SCAN_FAILURE_MARKER};

/// Runs the TUI with async scanning and returns the user's selected action and final state.
///
/// # Errors
///
/// Returns an I/O error if terminal operations fail.
pub fn run_tui_with_scan(
    config: &Config,
    rx: Receiver<ScanMessage>,
    _cancel_flag: Arc<AtomicBool>,
) -> io::Result<(Option<UIEvent>, AppState)> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut state = AppState::new_loading();

    let result = run_app_with_loading(&mut terminal, &mut state, rx, config);

    // Caller is responsible for cancelling thread and waiting for join

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result.map(|event| (event, state))
}

// Clippy suggests taking `&Receiver` here, but the event loop needs to own
// the `Receiver<ScanMessage>` and consume it (calling `try_recv` in a loop),
// so we intentionally pass it by value and suppress `needless_pass_by_value`.
#[allow(clippy::needless_pass_by_value)]
fn run_app_with_loading(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    state: &mut AppState,
    rx: Receiver<ScanMessage>,
    config: &Config,
) -> io::Result<Option<UIEvent>> {
    loop {
        terminal.draw(|f| view::render(f, state))?;

        // Check for scan messages
        if let Ok(msg) = rx.try_recv() {
            match msg {
                ScanMessage::Progress(message) => {
                    state.set_loading_message(message);
                }
                ScanMessage::ScanWarning(warning) => {
                    state.add_scan_warning(warning);
                }
                ScanMessage::Complete(packages) => {
                    if packages.is_empty() {
                        state.set_no_updates();
                    } else {
                        state.set_packages(packages, &config.exclude.permanent);
                    }
                }
            }
        }

        // Poll for keyboard events with timeout
        if event::poll(Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
        {
            if let Some(event) = controller::handle_key_event(state, key.code) {
                return Ok(Some(event));
            }
        }
    }
}

/// Runs the TUI for dependency conflict confirmation modal only.
/// State must already have `dependency_conflicts` set and `show_dependency_warning` = true.
///
/// # Errors
///
/// Returns an I/O error if terminal operations fail.
pub fn run_tui_for_confirmation(state: &mut AppState) -> io::Result<Option<UIEvent>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_modal_loop(&mut terminal, state);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_modal_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    state: &mut AppState,
) -> io::Result<Option<UIEvent>> {
    loop {
        terminal.draw(|frame| view::render(frame, state))?;

        if event::poll(Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
        {
            match controller::handle_dependency_warning_modal(state, key.code) {
                controller::ModalResult::Proceed(event) => return Ok(event),
                controller::ModalResult::Cancel => return Ok(None),
                controller::ModalResult::Quit => return Ok(Some(UIEvent::Quit)),
                controller::ModalResult::IgnoreKey => {}
            }
        }
    }
}
