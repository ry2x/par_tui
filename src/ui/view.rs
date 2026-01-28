use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

use super::app::{AppState, LoadingState};
use crate::models::package::PackageRepository;

pub fn render(frame: &mut Frame, state: &AppState) {
    match &state.loading_state {
        LoadingState::Scanning => render_loading(frame, state),
        LoadingState::Ready => render_main(frame, state),
        LoadingState::NoUpdates => render_no_updates(frame),
        LoadingState::Error(err) => render_error(frame, err),
    }
}

fn render_loading(frame: &mut Frame, state: &AppState) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Length(3),
            Constraint::Percentage(40),
        ])
        .split(area);

    let spinner = get_spinner();
    let text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(spinner, Style::default().fg(Color::Cyan)),
            Span::raw("  "),
            Span::raw(&state.loading_message),
        ]),
        Line::from(""),
    ];

    let paragraph = Paragraph::new(text).alignment(Alignment::Center).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Scanning for Updates"),
    );

    frame.render_widget(paragraph, chunks[1]);
}

fn render_no_updates(frame: &mut Frame) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Length(3),
            Constraint::Percentage(40),
        ])
        .split(area);

    let text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "✓ System is up to date!",
            Style::default().fg(Color::Green),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Press q to quit",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let paragraph = Paragraph::new(text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

    frame.render_widget(paragraph, chunks[1]);
}

fn render_error(frame: &mut Frame, error: &str) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Length(5),
            Constraint::Percentage(40),
        ])
        .split(area);

    let text = vec![
        Line::from(""),
        Line::from(Span::styled("✗ Error", Style::default().fg(Color::Red))),
        Line::from(""),
        Line::from(error),
        Line::from(""),
        Line::from(Span::styled(
            "Press q to quit",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let paragraph = Paragraph::new(text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("Scan Failed"));

    frame.render_widget(paragraph, chunks[1]);
}

fn get_spinner() -> &'static str {
    const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let frame = (now / 80) % SPINNER_FRAMES.len() as u128;
    SPINNER_FRAMES[frame as usize]
}

fn render_main(frame: &mut Frame, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(2),
            Constraint::Length(1),
        ])
        .split(frame.area());

    render_header(frame, chunks[0], state);
    render_package_list(frame, chunks[1], state);
    render_status(frame, chunks[2], state);
    render_keybinds(frame, chunks[3]);

    if state.show_help {
        render_help_modal(frame);
    }
}

fn render_header(frame: &mut Frame, area: Rect, state: &AppState) {
    let title = format!(
        "par_tui - [Updates Found: {}]                     [Help: ?]",
        state.packages.len()
    );
    let header = Paragraph::new(title).style(
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    );
    frame.render_widget(header, area);
}

fn render_package_list(frame: &mut Frame, area: Rect, state: &AppState) {
    let items: Vec<ListItem> = state
        .packages
        .iter()
        .enumerate()
        .map(|(idx, item)| {
            let checkbox = if item.is_temporarily_ignored || item.is_permanently_ignored {
                "[x]"
            } else {
                "[ ]"
            };

            let cursor = if idx == state.cursor_position {
                "> "
            } else {
                "  "
            };

            let (badge, badge_color) = match item.package.repository {
                PackageRepository::Official => ("Official", Color::Blue),
                PackageRepository::Aur => ("AUR", Color::Yellow),
            };

            let perm_marker = if item.is_permanently_ignored {
                " (PERM)"
            } else {
                ""
            };

            let version_info = format!(
                "{:12} -> {}",
                item.package.current_version.as_deref().unwrap_or("?"),
                item.package.new_version
            );

            let line = Line::from(vec![
                Span::raw(cursor),
                Span::raw(checkbox),
                Span::raw(" ["),
                Span::styled(badge, Style::default().fg(badge_color)),
                Span::raw("] "),
                Span::raw(format!("{:20} ", item.package.name)),
                Span::raw(version_info),
                Span::raw(perm_marker),
            ]);

            let style = if idx == state.cursor_position {
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::REVERSED)
            } else {
                Style::default()
            };

            ListItem::new(line).style(style)
        })
        .collect();

    let list = List::new(items).block(Block::default().borders(Borders::ALL));
    frame.render_widget(list, area);
}

fn render_status(frame: &mut Frame, area: Rect, state: &AppState) {
    let (official, aur, ignored) = state.stats();

    let status_lines = vec![
        Line::from("Mode: Entire System (paru)".to_string()),
        Line::from(format!(
            "Stats: Official ({official}) | AUR ({aur}) | To Ignore: {ignored}"
        )),
    ];

    let status = Paragraph::new(status_lines).block(Block::default().borders(Borders::ALL));
    frame.render_widget(status, area);
}

fn render_keybinds(frame: &mut Frame, area: Rect) {
    let keybinds =
        Paragraph::new("[Enter] Entire  [o] Official  [Space] Toggle  [p] Perm  [q] Quit")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(keybinds, area);
}

fn render_help_modal(frame: &mut Frame) {
    let area = centered_rect(60, 50, frame.area());

    let help_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("[Enter]   ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": Update Entire System (paru)"),
        ]),
        Line::from(vec![
            Span::styled("[o]       ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": Update Official Only (pacman)"),
        ]),
        Line::from(vec![
            Span::styled("[Space]   ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": Toggle Temporary Ignore"),
        ]),
        Line::from(vec![
            Span::styled("[p]       ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": Toggle Permanent Ignore"),
        ]),
        Line::from(vec![
            Span::styled("[j/k]     ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": Navigate List"),
        ]),
        Line::from(vec![
            Span::styled("[?]       ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": Toggle This Help"),
        ]),
        Line::from(vec![
            Span::styled("[q]       ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": Quit Application"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw(" GitHub: "),
            Span::styled(
                "https://github.com/ry2x/par_tui",
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::UNDERLINED),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Press ? to close",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let help = Paragraph::new(help_text)
        .block(
            Block::default()
                .title("HELP")
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::Black)),
        )
        .alignment(Alignment::Left);

    frame.render_widget(Clear, area);
    frame.render_widget(help, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
