use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

use crate::ui::{app::{AppState, UIEvent}, view};

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
                (KeyCode::Char('j'), _) | (KeyCode::Down, _) => state.move_cursor_down(),
                (KeyCode::Char('k'), _) | (KeyCode::Up, _) => state.move_cursor_up(),
                (KeyCode::Char(' '), _) => state.toggle_current_package(),
                (KeyCode::Enter, KeyModifiers::SHIFT) => {
                    return Ok(Some(UIEvent::UpdateOfficialOnly));
                }
                (KeyCode::Enter, _) => {
                    return Ok(Some(UIEvent::UpdateEntireSystem));
                }
                _ => {}
            }
        }
    }
}
