use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::thread::{self, JoinHandle};
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

/// Scan failure markers for warning messages
pub const OFFICIAL_SCAN_FAILURE_MARKER: &str = "Official";
pub const AUR_SCAN_FAILURE_MARKER: &str = "AUR";

pub enum ScanMessage {
    Progress(String),
    ScanWarning(String),
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
    let cancel_flag = Arc::new(AtomicBool::new(false));
    let scan_handle = start_scan_thread(tx, has_paru, Arc::clone(&cancel_flag));

    let result = run_app_with_loading(&mut terminal, &mut state, rx, config);

    // Signal thread to stop if still running
    cancel_flag.store(true, Ordering::Relaxed);

    // Wait for thread to complete and detect panics
    if scan_handle.join().is_err() {
        eprintln!("Warning: Scan thread panicked during execution.");
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result.map(|event| (event, state))
}

fn start_scan_thread(
    tx: Sender<ScanMessage>,
    has_paru: bool,
    cancel_flag: Arc<AtomicBool>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        // Helper macro to send message and return early if channel is closed
        macro_rules! send_or_return {
            ($msg:expr) => {
                if tx.send($msg).is_err() {
                    return;
                }
            };
        }

        let mut all_packages = Vec::new();
        let mut official_failed = false;
        let mut aur_failed = false;

        // Scan official packages
        if cancel_flag.load(Ordering::Relaxed) {
            return;
        }

        send_or_return!(ScanMessage::Progress(
            "Scanning official repositories...".to_string()
        ));

        let tx_clone = tx.clone();
        match command::run_checkupdates_with_callback(|attempt, max| {
            let _ = tx_clone.send(ScanMessage::Progress(format!(
                "Retrying checkupdates (attempt {attempt}/{max})"
            )));
        }) {
            Ok(output) => {
                let packages = pacman::parse_checkupdates_output(&output);
                let count = packages.len();
                send_or_return!(ScanMessage::Progress(format!(
                    "Found {} official update{}",
                    count,
                    if count == 1 { "" } else { "s" }
                )));
                all_packages.extend(packages);
            },
            Err(e) => {
                official_failed = true;
                send_or_return!(ScanMessage::Progress(format!(
                    "Warning: Could not scan official repos: {e:?}"
                )));
            },
        }

        // Scan AUR packages
        if has_paru && !cancel_flag.load(Ordering::Relaxed) {
            send_or_return!(ScanMessage::Progress(
                "Scanning AUR packages...".to_string()
            ));

            match command::run_paru_query_aur() {
                Ok(output) => {
                    let packages = paru::parse_paru_output(&output);
                    let count = packages.len();
                    send_or_return!(ScanMessage::Progress(format!(
                        "Found {} AUR update{}",
                        count,
                        if count == 1 { "" } else { "s" }
                    )));
                    all_packages.extend(packages);
                },
                Err(e) => {
                    aur_failed = true;
                    send_or_return!(ScanMessage::Progress(format!(
                        "Warning: Could not scan AUR packages: {e:?}"
                    )));
                },
            }
        }

        // Check if cancelled before sending final messages
        if cancel_flag.load(Ordering::Relaxed) {
            return;
        }

        // Final status message
        let total = all_packages.len();
        send_or_return!(ScanMessage::Progress(format!(
            "Scan complete. Total: {} update{}",
            total,
            if total == 1 { "" } else { "s" }
        )));

        // Send warning about scan failures
        if official_failed || aur_failed {
            let mut failed_sources = Vec::new();
            if official_failed {
                failed_sources.push(OFFICIAL_SCAN_FAILURE_MARKER);
            }
            if aur_failed {
                failed_sources.push(AUR_SCAN_FAILURE_MARKER);
            }
            send_or_return!(ScanMessage::ScanWarning(format!(
                "{} scan failed",
                failed_sources.join(" & ")
            )));
        }

        send_or_return!(ScanMessage::Complete(all_packages));
    })
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
                },
                ScanMessage::ScanWarning(warning) => {
                    state.add_scan_warning(warning);
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
        if event::poll(Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
        {
            match (&state.loading_state, key.code) {
                // Allow quit in any state
                (_, KeyCode::Char('q')) => return Ok(Some(UIEvent::Quit)),

                // Allow reload if official scan failed
                (LoadingState::Ready | LoadingState::NoUpdates, KeyCode::Char('r'))
                    if state.has_official_scan_failed() =>
                {
                    return Ok(Some(UIEvent::Reload));
                },

                // Only allow other keys when ready
                (LoadingState::Ready, KeyCode::Char('?')) => state.toggle_help(),
                (LoadingState::Ready, KeyCode::Char('j') | KeyCode::Down) => {
                    state.move_cursor_down();
                },
                (LoadingState::Ready, KeyCode::Char('k') | KeyCode::Up) => {
                    state.move_cursor_up();
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
