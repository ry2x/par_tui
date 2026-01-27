use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

use super::app::{AppState, PackageItem};
use crate::models::package::PackageRepository;

pub fn render(frame: &mut Frame, state: &AppState) {
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
    let header = Paragraph::new(title)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
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
                Style::default().fg(Color::White).add_modifier(Modifier::REVERSED)
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
        Line::from(format!("Mode: Entire System (paru)")),
        Line::from(format!(
            "Stats: Official ({}) | AUR ({}) | To Ignore: {}",
            official, aur, ignored
        )),
    ];

    let status = Paragraph::new(status_lines).block(Block::default().borders(Borders::ALL));
    frame.render_widget(status, area);
}

fn render_keybinds(frame: &mut Frame, area: Rect) {
    let keybinds = Paragraph::new("[Enter] Entire  [S-Enter] Official Only  [Space] Toggle  [q] Quit")
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
            Span::styled("[S-Enter] ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": Update Official Only (pacman)"),
        ]),
        Line::from(vec![
            Span::styled("[Space]   ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": Toggle Package Ignore"),
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
