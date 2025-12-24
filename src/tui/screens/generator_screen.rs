use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::PasswordMode;
use crate::tui::app::App;

pub fn render_generator_screen(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(frame.area());

    let title = Paragraph::new("Password Generator")
        .block(Block::default().borders(Borders::ALL).title("Generator"))
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);
    frame.render_widget(title, chunks[0]);

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(chunks[1]);

    render_mode_selection(frame, app, main_chunks[0]);
    render_parameters(frame, app, main_chunks[1]);

    let footer_text = if app.error_message.is_some() {
        format!("Error: {}", app.error_message.as_ref().unwrap())
    } else if !app.generated_passwords.is_empty() {
        format!("Generated {} password(s)", app.generated_passwords.len())
    } else {
        "Use ←→ to change mode, ↑↓ to navigate, Enter to generate, Esc to go back".to_string()
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

fn render_mode_selection(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let modes = [
        ("Random", PasswordMode::Random),
        ("Pattern", PasswordMode::Pattern),
        ("Phrase", PasswordMode::Phrase),
        ("Deterministic", PasswordMode::Deterministic),
    ];

    let items: Vec<ListItem> = modes
        .iter()
        .enumerate()
        .map(|(idx, (name, mode))| {
            let prefix = if app.generator.selected_mode_index == idx {
                "✓ "
            } else {
                "  "
            };
            let style = if app.generator.mode == *mode {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(format!("{}{}", prefix, name)).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Mode"))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    let mut state = ratatui::widgets::ListState::default();
    state.select(Some(app.generator.selected_mode_index));
    frame.render_stateful_widget(list, area, &mut state);
}

fn render_parameters(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let mut items = Vec::new();

    match app.generator.mode {
        PasswordMode::Random => {
            items.push((format!("Length: {}", app.generator.length), 1));
            items.push((format!("Count: {}", app.generator.count), 2));
        }
        PasswordMode::Pattern => {
            items.push((
                format!(
                    "Pattern: {}",
                    if app.generator.pattern.is_empty() {
                        "<Enter pattern (U=Upper, L=Lower, D=Digit, S=Special)>".to_string()
                    } else {
                        app.generator.pattern.clone()
                    }
                ),
                1,
            ));
            items.push((format!("Count: {}", app.generator.count), 2));
        }
        PasswordMode::Phrase => {
            items.push((format!("Words: {}", app.generator.words.unwrap_or(4)), 1));
            items.push((format!("Count: {}", app.generator.count), 2));
        }
        PasswordMode::Deterministic => {
            items.push((
                format!(
                    "Seed Env Var: {}",
                    if app.generator.seed_env.is_empty() {
                        "<Enter env var name>".to_string()
                    } else {
                        app.generator.seed_env.clone()
                    }
                ),
                1,
            ));
            items.push((
                format!(
                    "Service: {}",
                    if app.generator.service.is_empty() {
                        "<Optional>".to_string()
                    } else {
                        app.generator.service.clone()
                    }
                ),
                2,
            ));
            items.push((
                format!(
                    "Salt: {}",
                    if app.generator.salt.is_empty() {
                        "<Optional>".to_string()
                    } else {
                        app.generator.salt.clone()
                    }
                ),
                3,
            ));
            items.push((format!("Count: {}", app.generator.count), 4));
        }
    }

    let list_items: Vec<ListItem> = items
        .iter()
        .map(|(text, idx)| {
            let prefix = if app.selected_index == *idx {
                ">> "
            } else {
                "   "
            };
            let style = if app.selected_index == *idx {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(format!("{}{}", prefix, text)).style(style)
        })
        .collect();

    let mut all_lines = Vec::new();
    all_lines.extend(list_items);

    if !app.generated_passwords.is_empty() {
        all_lines.push(ListItem::new(""));
        all_lines.push(ListItem::new("Generated Passwords:"));
        all_lines.push(ListItem::new("─".repeat(40)));
        for (idx, password) in app.generated_passwords.iter().enumerate() {
            all_lines.push(ListItem::new(format!("{}. {}", idx + 1, password)));
        }
    }

    let list = List::new(all_lines)
        .block(Block::default().borders(Borders::ALL).title("Parameters"))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    let mut state = ratatui::widgets::ListState::default();
    if app.selected_index > 0 {
        let list_index = items.iter().position(|(_, idx)| *idx == app.selected_index);
        if let Some(idx) = list_index {
            state.select(Some(idx));
        }
    }
    frame.render_stateful_widget(list, area, &mut state);
}
