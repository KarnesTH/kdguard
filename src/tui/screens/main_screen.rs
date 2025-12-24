use lingua_i18n_rs::prelude::Lingua;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::tui::app::App;

pub fn render_main_screen(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(frame.area());

    let title = Paragraph::new(
        Lingua::t("cli.about", &[]).unwrap_or_else(|_| "kdguard - Password Generator".to_string()),
    )
    .block(Block::default().borders(Borders::ALL).title("kdguard"))
    .style(
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )
    .alignment(Alignment::Center);
    frame.render_widget(title, chunks[0]);

    let menu_items = vec![
        ListItem::new(
            Lingua::t("tui.main.generate", &[])
                .unwrap_or_else(|_| "Generate Passwords".to_string()),
        ),
        ListItem::new(
            Lingua::t("tui.main.check", &[]).unwrap_or_else(|_| "Check Password".to_string()),
        ),
        ListItem::new(
            Lingua::t("tui.main.settings", &[]).unwrap_or_else(|_| "Settings".to_string()),
        ),
        ListItem::new(Lingua::t("tui.main.help", &[]).unwrap_or_else(|_| "Help".to_string())),
        ListItem::new(Lingua::t("tui.main.exit", &[]).unwrap_or_else(|_| "Exit".to_string())),
    ];

    let menu = List::new(menu_items)
        .block(Block::default().borders(Borders::ALL).title("Menu"))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    frame.render_stateful_widget(
        menu,
        chunks[1],
        &mut ratatui::widgets::ListState::default().with_selected(Some(app.selected_index)),
    );

    let footer = Paragraph::new("Use ↑↓ to navigate, Enter to select, q/Esc to exit")
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);
    frame.render_widget(footer, chunks[2]);
}
