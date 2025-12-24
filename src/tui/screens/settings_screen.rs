use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::tui::app::App;

pub fn render_settings_screen(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(frame.area());

    let title = Paragraph::new("Settings")
        .block(Block::default().borders(Borders::ALL).title("Settings"))
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);
    frame.render_widget(title, chunks[0]);

    let items = vec![
        ListItem::new(format!("Language: {}", app.settings.language)),
        ListItem::new(format!("Default Length: {}", app.settings.default_length)),
        ListItem::new(format!("Default Count: {}", app.settings.default_count)),
        ListItem::new(format!(
            "Auto Save: {}",
            if app.settings.auto_save { "Yes" } else { "No" }
        )),
    ];

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Configuration"),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    let mut state = ratatui::widgets::ListState::default();
    state.select(Some(app.settings.selected_index));
    frame.render_stateful_widget(list, chunks[1], &mut state);

    let footer_text = if app.error_message.is_some() {
        format!("Error: {}", app.error_message.as_ref().unwrap())
    } else {
        "Use ↑↓ to navigate, ←→ to change values, Enter to save, Esc to go back".to_string()
    };

    let footer = Paragraph::new(footer_text)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(if app.error_message.is_some() {
            Color::Red
        } else {
            Color::Gray
        }))
        .alignment(Alignment::Center);
    frame.render_widget(footer, chunks[2]);
}
