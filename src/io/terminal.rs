use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

use crate::models::config::Config;
use crate::models::package::Package;
use crate::ui::{
    app::{AppState, LoadingState, UIEvent},
    view,
};
use crate::{
    io::command,
    parser::{pacman, paru},
};

pub enum ScanMessage {
    Progress(String),
    OfficialComplete(Result<Vec<Package>, String>),
    AurComplete(Result<Vec<Package>, String>),
    Complete(Vec<Package>),
}

/// Runs the TUI with async scanning and returns the user's selected action and final state.
///
/// # Errors
///
/// Returns an I/O error if terminal operations fail.
pub fn run_tui_with_scan(
    config: &Config,
    has_paru: bool,
) -> io::Result<(Option<UIEvent>, AppState)> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut state = AppState::new_loading();

    let (tx, rx) = mpsc::channel();
    start_scan_thread(tx, has_paru);

    let result = run_app_with_loading(&mut terminal, &mut state, rx, config);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result.map(|event| (event, state))
}

fn start_scan_thread(tx: Sender<ScanMessage>, has_paru: bool) {
    thread::spawn(move || {
        let mut all_packages = Vec::new();

        // Scan official packages
        let _ = tx.send(ScanMessage::Progress(
            "Scanning official repositories...".to_string(),
        ));

        let tx_clone = tx.clone();
        match command::run_checkupdates_with_callback(|attempt, max| {
            let _ = tx_clone.send(ScanMessage::Progress(format!(
                "Retrying checkupdates (attempt {}/{})",
                attempt + 1,
                max
            )));
        }) {
            Ok(output) => {
                let packages = pacman::parse_checkupdates_output(&output);
                let count = packages.len();
                let _ = tx.send(ScanMessage::OfficialComplete(Ok(packages.clone())));
                let _ = tx.send(ScanMessage::Progress(format!(
                    "Found {} official update{}",
                    count,
                    if count == 1 { "" } else { "s" }
                )));
                all_packages.extend(packages);
            },
            Err(e) => {
                let _ = tx.send(ScanMessage::OfficialComplete(Err(format!("{e:?}"))));
                let _ = tx.send(ScanMessage::Progress(
                    "Warning: Could not scan official repos".to_string(),
                ));
            },
        }

        // Scan AUR packages
        if has_paru {
            let _ = tx.send(ScanMessage::Progress(
                "Scanning AUR packages...".to_string(),
            ));

            match command::run_paru_query_aur() {
                Ok(output) => {
                    let packages = paru::parse_paru_output(&output);
                    let count = packages.len();
                    let _ = tx.send(ScanMessage::AurComplete(Ok(packages.clone())));
                    let _ = tx.send(ScanMessage::Progress(format!(
                        "Found {} AUR update{}",
                        count,
                        if count == 1 { "" } else { "s" }
                    )));
                    all_packages.extend(packages);
                },
                Err(e) => {
                    let _ = tx.send(ScanMessage::AurComplete(Err(format!("{e:?}"))));
                    let _ = tx.send(ScanMessage::Progress(
                        "Warning: Could not scan AUR packages".to_string(),
                    ));
                },
            }
        }

        // Final message
        let total = all_packages.len();
        let _ = tx.send(ScanMessage::Progress(format!(
            "Scan complete. Total: {} update{}",
            total,
            if total == 1 { "" } else { "s" }
        )));

        let _ = tx.send(ScanMessage::Complete(all_packages));
    });
}

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
                },
                ScanMessage::OfficialComplete(_) | ScanMessage::AurComplete(_) => {
                    // Progress updates, continue
                },
                ScanMessage::Complete(packages) => {
                    if packages.is_empty() {
                        state.set_no_updates();
                    } else {
                        state.set_packages(packages, &config.exclude.permanent);
                    }
                },
            }
        }

        // Poll for keyboard events with timeout
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match (&state.loading_state, key.code) {
                    // Allow quit in any state
                    (_, KeyCode::Char('q')) => return Ok(Some(UIEvent::Quit)),

                    // Only allow other keys when ready
                    (LoadingState::Ready, KeyCode::Char('?')) => state.toggle_help(),
                    (LoadingState::Ready, KeyCode::Char('j') | KeyCode::Down) => {
                        state.move_cursor_down()
                    },
                    (LoadingState::Ready, KeyCode::Char('k') | KeyCode::Up) => {
                        state.move_cursor_up()
                    },
                    (LoadingState::Ready, KeyCode::Char('p')) => state.toggle_permanent_ignore(),
                    (LoadingState::Ready, KeyCode::Char(' ')) => state.toggle_current_package(),
                    (LoadingState::Ready, KeyCode::Char('o')) => {
                        return Ok(Some(UIEvent::UpdateOfficialOnly));
                    },
                    (LoadingState::Ready, KeyCode::Enter) => {
                        return Ok(Some(UIEvent::UpdateEntireSystem));
                    },
                    _ => {},
                }
            }
        }
    }
}
