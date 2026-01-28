use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;

use crate::ui::{
    app::{AppState, UIEvent},
    view,
};

/// Runs the TUI and returns the user's selected action and final state.
///
/// # Errors
///
/// Returns an I/O error if terminal operations fail.
pub fn run_tui(mut state: AppState) -> io::Result<(Option<UIEvent>, AppState)> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal, &mut state);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result.map(|event| (event, state))
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    state: &mut AppState,
) -> io::Result<Option<UIEvent>> {
    loop {
        terminal.draw(|f| view::render(f, state))?;

        if let Event::Key(key) = event::read()? {
            match (key.code, key.modifiers) {
                (KeyCode::Char('q'), _) => return Ok(Some(UIEvent::Quit)),
                (KeyCode::Char('?'), _) => state.toggle_help(),
                (KeyCode::Char('j') | KeyCode::Down, _) => state.move_cursor_down(),
                (KeyCode::Char('k') | KeyCode::Up, _) => state.move_cursor_up(),
                (KeyCode::Char('p'), _) => state.toggle_permanent_ignore(),
                (KeyCode::Char(' '), _) => state.toggle_current_package(),
                (KeyCode::Enter, KeyModifiers::SHIFT) => {
                    return Ok(Some(UIEvent::UpdateOfficialOnly));
                },
                (KeyCode::Enter, _) => {
                    return Ok(Some(UIEvent::UpdateEntireSystem));
                },
                _ => {},
            }
        }
    }
}
