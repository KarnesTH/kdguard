use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

use crate::tui::app::App;

const HIGHLIGHT_SYMBOL: &str = ">> ";
const FOOTER_HELP_TEXT: &str =
    "Use ↑↓ to navigate, ←→ to change values, Enter to save, Esc to go back";

pub fn render_settings_screen(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(frame.area());

    render_title(frame, chunks[0]);
    render_configuration_list(frame, app, chunks[1]);
    render_footer(frame, app, chunks[2]);
}

fn render_title(frame: &mut Frame, area: Rect) {
    let title = Paragraph::new("Settings")
        .block(Block::default().borders(Borders::ALL).title("Settings"))
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);
    frame.render_widget(title, area);
}

fn render_configuration_list(frame: &mut Frame, app: &App, area: Rect) {
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
        .highlight_symbol(HIGHLIGHT_SYMBOL);

    let mut state = ListState::default();
    state.select(Some(app.settings.selected_index));
    frame.render_stateful_widget(list, area, &mut state);
}

fn render_footer(frame: &mut Frame, app: &App, area: Rect) {
    let (footer_text, footer_style) = if let Some(error) = &app.error_message {
        (format!("Error: {}", error), Style::default().fg(Color::Red))
    } else {
        (
            FOOTER_HELP_TEXT.to_string(),
            Style::default().fg(Color::Gray),
        )
    };

    let footer = Paragraph::new(footer_text)
        .block(Block::default().borders(Borders::ALL))
        .style(footer_style)
        .alignment(Alignment::Center);
    frame.render_widget(footer, area);
}
