use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::PasswordMode;
use crate::tui::app::App;

pub fn render_generator_mode_selection(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(frame.area());

    let title = Paragraph::new("Select Generator Mode")
        .block(Block::default().borders(Borders::ALL).title("Generator"))
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);
    frame.render_widget(title, chunks[0]);

    let modes = [
        ("Random", "Generates passwords with random characters"),
        ("Pattern", "Generates passwords based on a custom pattern"),
        ("Phrase", "Generates memorable passphrases from word lists"),
        (
            "Deterministic",
            "Generates consistent passwords from a seed",
        ),
    ];

    let items: Vec<ListItem> = modes
        .iter()
        .enumerate()
        .map(|(idx, (name, desc))| {
            let style = if app.generator.selected_mode_index == idx {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            let content = format!("{:<15} - {}", name, desc);
            ListItem::new(content).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Available Modes"),
        )
        .highlight_symbol(">> ");

    frame.render_widget(list, chunks[1]);

    let footer = Paragraph::new("↑↓ to navigate, Enter to select, Esc to go back")
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);
    frame.render_widget(footer, chunks[2]);
}

pub fn render_generator_screen(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(frame.area());

    let mode_name = match app.generator.mode {
        PasswordMode::Random => "Random",
        PasswordMode::Pattern => "Pattern",
        PasswordMode::Phrase => "Phrase",
        PasswordMode::Deterministic => "Deterministic",
    };

    let title = Paragraph::new(format!("Password Generator: {}", mode_name))
        .block(Block::default().borders(Borders::ALL).title("Generator"))
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);
    frame.render_widget(title, chunks[0]);

    render_parameters(frame, app, chunks[1]);

    let footer_text = if app.error_message.is_some() {
        format!("Error: {}", app.error_message.as_ref().unwrap())
    } else if !app.generated_passwords.is_empty() {
        format!("Generated {} password(s)", app.generated_passwords.len())
    } else {
        "↑↓ to navigate, ←→ change values, Enter to generate, Esc to change mode".to_string()
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

fn render_parameters(frame: &mut Frame, app: &App, area: Rect) {
    let mut items = Vec::new();

    match app.generator.mode {
        PasswordMode::Random => {
            items.push((format!("Length: {}", app.generator.length), 0));
            items.push((format!("Count: {}", app.generator.count), 1));
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
                0,
            ));
            items.push((format!("Count: {}", app.generator.count), 1));
        }
        PasswordMode::Phrase => {
            items.push((format!("Words: {}", app.generator.words.unwrap_or(4)), 0));
            items.push((format!("Count: {}", app.generator.count), 1));
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
                0,
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
                1,
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
                2,
            ));
            items.push((format!("Count: {}", app.generator.count), 3));
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
    let list_index = items.iter().position(|(_, idx)| *idx == app.selected_index);
    if let Some(idx) = list_index {
        state.select(Some(idx));
    }

    frame.render_stateful_widget(list, area, &mut state);
}
