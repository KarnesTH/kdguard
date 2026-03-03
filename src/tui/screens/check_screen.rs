use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Gauge, Paragraph, Wrap},
};

use crate::password::health_check::PasswordAnalysis;
use crate::tui::app::{App, InputMode};

const CHECK_TITLE: &str = "Password Health Check";
const CHECK_BLOCK_TITLE: &str = "Check Password";
const PASSWORD_FIELD_TITLE: &str = "Password";
const ANALYSIS_FIELD_TITLE: &str = "Analysis";
const PLACEHOLDER_TEXT: &str = "Enter a password and press Enter to check its strength";
const FOOTER_TEXT: &str =
    "Type password, Enter to check, 'd' for details, 'c' to clear, Esc to go back";

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

    render_header(frame, chunks[0]);
    render_password_input(frame, app, chunks[1]);
    render_analysis_area(frame, app, chunks[2]);
    render_footer(frame, chunks[3]);
}

fn render_header(frame: &mut Frame, area: Rect) {
    let title = Paragraph::new(CHECK_TITLE)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(CHECK_BLOCK_TITLE),
        )
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);
    frame.render_widget(title, area);
}

fn render_password_input(frame: &mut Frame, app: &App, area: Rect) {
    let input_display = if app.password_input.is_empty() {
        format!("<{}>", PLACEHOLDER_TEXT)
    } else {
        "*".repeat(app.password_input.len())
    };

    let input_style = if app.input_mode == InputMode::Editing {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    let input = Paragraph::new(input_display)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(PASSWORD_FIELD_TITLE),
        )
        .style(input_style);
    frame.render_widget(input, area);
}

fn render_analysis_area(frame: &mut Frame, app: &App, area: Rect) {
    if let Some(ref analysis) = app.check_result {
        render_analysis_details(frame, app, area, analysis);
    } else {
        let placeholder = Paragraph::new(PLACEHOLDER_TEXT)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(ANALYSIS_FIELD_TITLE),
            )
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Gray));
        frame.render_widget(placeholder, area);
    }
}

fn render_analysis_details(frame: &mut Frame, app: &App, area: Rect, analysis: &PasswordAnalysis) {
    let rating_color = match analysis.rating.as_str() {
        s if s.contains("Weak") => Color::Red,
        s if s.contains("Medium") => Color::Yellow,
        s if s.contains("Strong") => Color::Green,
        _ => Color::Green,
    };

    let mut lines = vec![
        format!("Score: {}/100", analysis.score.total),
        format!("Length: {}", analysis.length),
        format!("Entropy: {:.2}", analysis.entropy),
        "".to_string(),
        "Requirements:".to_string(),
        format!(
            "  Lowercase: {}",
            if analysis.has_lowercase { "✓" } else { "✗" }
        ),
        format!(
            "  Uppercase: {}",
            if analysis.has_uppercase { "✓" } else { "✗" }
        ),
        format!(
            "  Digits:    {}",
            if analysis.has_digit { "✓" } else { "✗" }
        ),
        format!(
            "  Special:   {}",
            if analysis.has_special { "✓" } else { "✗" }
        ),
    ];

    if app.show_detailed_check {
        lines.push("".to_string());
        lines.push("Detailed Scores:".to_string());
        lines.push(format!(
            "  Length Score:     {}",
            analysis.score.length_score
        ));
        lines.push(format!(
            "  Diversity Score:  {}",
            analysis.score.diversity_score
        ));
        lines.push(format!(
            "  Complexity Score: {}",
            analysis.score.complexity_score
        ));
        lines.push(format!(
            "  Entropy Score:    {}",
            analysis.score.entropy_score
        ));

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
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(ANALYSIS_FIELD_TITLE),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(content, area);

    // Render Gauge at the bottom of the analysis area
    let gauge_area = Rect::new(area.x + 1, area.y + area.height - 2, area.width - 2, 1);

    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::NONE))
        .gauge_style(Style::default().fg(rating_color))
        .percent(analysis.score.total.min(100) as u16)
        .label(format!(
            "Rating: {} ({}%)",
            analysis.rating, analysis.score.total
        ));
    frame.render_widget(gauge, gauge_area);
}

fn render_footer(frame: &mut Frame, area: Rect) {
    let footer = Paragraph::new(FOOTER_TEXT)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);
    frame.render_widget(footer, area);
}
