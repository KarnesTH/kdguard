use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Gauge, Paragraph},
};

use crate::tui::app::App;

pub fn render_check_screen(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(5),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(frame.area());

    let title = Paragraph::new("Password Health Check")
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Check Password"),
        )
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);
    frame.render_widget(title, chunks[0]);

    let input_display = if app.password_input.is_empty() {
        "<Enter password to check>".to_string()
    } else {
        "*".repeat(app.password_input.len())
    };

    let input_style = if app.input_mode == crate::tui::app::InputMode::Editing {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    let input = Paragraph::new(input_display)
        .block(Block::default().borders(Borders::ALL).title("Password"))
        .style(input_style);
    frame.render_widget(input, chunks[1]);

    if let Some(ref analysis) = app.check_result {
        render_analysis(frame, app, chunks[2], analysis);
    } else {
        let placeholder = Paragraph::new("Enter a password and press Enter to check its strength")
            .block(Block::default().borders(Borders::ALL).title("Analysis"))
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Gray));
        frame.render_widget(placeholder, chunks[2]);
    }

    let footer = Paragraph::new("Type password, Enter to check, 'd' for details, Esc to go back")
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);
    frame.render_widget(footer, chunks[3]);
}

fn render_analysis(
    frame: &mut Frame,
    app: &App,
    area: ratatui::layout::Rect,
    analysis: &crate::password::health_check::PasswordAnalysis,
) {
    let mut lines = Vec::new();

    let rating_color = match analysis.rating.as_str() {
        s if s.contains("Weak") => Color::Red,
        s if s.contains("Medium") => Color::Yellow,
        s if s.contains("Strong") => Color::Green,
        _ => Color::Green,
    };

    lines.push(format!("Rating: {}", analysis.rating));
    lines.push(format!("Score: {}/100", analysis.score.total));
    lines.push(format!("Length: {}", analysis.length));
    lines.push(format!("Entropy: {:.2}", analysis.entropy));
    lines.push("".to_string());
    lines.push("Requirements:".to_string());
    lines.push(format!(
        "  Lowercase: {}",
        if analysis.has_lowercase { "✓" } else { "✗" }
    ));
    lines.push(format!(
        "  Uppercase: {}",
        if analysis.has_uppercase { "✓" } else { "✗" }
    ));
    lines.push(format!(
        "  Digits: {}",
        if analysis.has_digit { "✓" } else { "✗" }
    ));
    lines.push(format!(
        "  Special: {}",
        if analysis.has_special { "✓" } else { "✗" }
    ));

    if app.show_detailed_check {
        lines.push("".to_string());
        lines.push("Detailed Scores:".to_string());
        lines.push(format!("  Length Score: {}", analysis.score.length_score));
        lines.push(format!(
            "  Diversity Score: {}",
            analysis.score.diversity_score
        ));
        lines.push(format!(
            "  Complexity Score: {}",
            analysis.score.complexity_score
        ));
        lines.push(format!("  Entropy Score: {}", analysis.score.entropy_score));

        if !analysis.warnings.is_empty() {
            lines.push("".to_string());
            lines.push("Warnings:".to_string());
            for warning in &analysis.warnings {
                lines.push(format!("  ⚠ {}", warning));
            }
        }

        if !analysis.suggestions.is_empty() {
            lines.push("".to_string());
            lines.push("Suggestions:".to_string());
            for suggestion in &analysis.suggestions {
                lines.push(format!("  💡 {}", suggestion));
            }
        }
    }

    let content = Paragraph::new(lines.join("\n"))
        .block(Block::default().borders(Borders::ALL).title("Analysis"))
        .wrap(ratatui::widgets::Wrap { trim: true });

    frame.render_widget(content, area);

    let gauge_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(100)])
        .split(area)[0];

    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::NONE))
        .gauge_style(Style::default().fg(rating_color))
        .percent(analysis.score.total.min(100) as u16)
        .label(format!("{}%", analysis.score.total));
    frame.render_widget(gauge, gauge_area);
}
