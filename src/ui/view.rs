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
            Constraint::Percentage(35),
            Constraint::Length(9),
            Constraint::Percentage(35),
        ])
        .split(area);

    let spinner = get_spinner();
    let text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                spinner,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(&state.loading_message, Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Please wait...",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Press [q] to quit",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
    ];

    let paragraph = Paragraph::new(text).alignment(Alignment::Center).block(
        Block::default().borders(Borders::ALL).title(Span::styled(
            "Scanning for Updates",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
    );

    frame.render_widget(paragraph, chunks[1]);
}

fn render_no_updates(frame: &mut Frame) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Length(6), // 4 lines + 2 borders
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
            Constraint::Length(8), // 6 lines + 2 borders
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
        .unwrap_or_default()
        .as_millis();
    // Match 100ms polling interval for smooth animation
    let frame = (now / 100) % SPINNER_FRAMES.len() as u128;
    SPINNER_FRAMES[frame as usize]
}

fn render_main(frame: &mut Frame, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Header
            Constraint::Min(0),    // Package list
            Constraint::Length(3), // Status (2 borders + 1 content line)
            Constraint::Length(1), // Keybinds
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
    let left_text = format!("par_tui - [Updates Found: {}]", state.packages.len());
    let right_text = "[Help: ?]";

    let available_space = area.width as usize;
    let padding = available_space
        .saturating_sub(left_text.len())
        .saturating_sub(right_text.len());

    let title = format!("{}{}{}", left_text, " ".repeat(padding), right_text);

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

    // Calculate scroll offset to keep cursor visible
    let visible_height = area.height.saturating_sub(2) as usize; // Subtract borders
    let total_items = state.packages.len();

    // Calculate offset to keep cursor visible
    let offset = if total_items <= visible_height {
        0
    } else if state.cursor_position < visible_height / 2 {
        0
    } else if state.cursor_position >= total_items - visible_height / 2 {
        total_items.saturating_sub(visible_height)
    } else {
        state.cursor_position.saturating_sub(visible_height / 2)
    };

    // Only render visible items
    let visible_items: Vec<ListItem> = items
        .into_iter()
        .skip(offset)
        .take(visible_height)
        .collect();

    let list = List::new(visible_items).block(Block::default().borders(Borders::ALL));
    frame.render_widget(list, area);
}

fn render_status(frame: &mut Frame, area: Rect, state: &AppState) {
    let (official, aur, ignored) = state.stats();

    let stats_text = format!("Stats: Official ({official}) | AUR ({aur}) | To Ignore: {ignored}");

    let status_line = if state.scan_warnings.is_empty() {
        stats_text
    } else {
        format!("{} | ⚠ {}", stats_text, state.scan_warnings.join(", "))
    };

    let status = Paragraph::new(status_line).block(Block::default().borders(Borders::ALL));
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
