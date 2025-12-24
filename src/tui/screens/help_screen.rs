use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph},
};

use crate::tui::app::App;

pub fn render_help_screen(frame: &mut Frame, _app: &App) {
    let chunks = Layout::default()
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(frame.area());

    let title = Paragraph::new("Help")
        .block(Block::default().borders(Borders::ALL).title("Help"))
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);
    frame.render_widget(title, chunks[0]);

    let help_text = vec![
        "Keyboard Shortcuts:".to_string(),
        "".to_string(),
        "Navigation:".to_string(),
        "  ↑↓     - Navigate up/down".to_string(),
        "  ←→     - Change values (in generator/settings)".to_string(),
        "  Enter  - Select/Confirm".to_string(),
        "  Esc    - Go back/Exit".to_string(),
        "  q      - Quit".to_string(),
        "".to_string(),
        "Generator Mode:".to_string(),
        "  Random        - Generate random passwords".to_string(),
        "  Pattern       - Generate from pattern (U=Upper, L=Lower, D=Digit, S=Special)"
            .to_string(),
        "  Phrase        - Generate passphrase from wordlist".to_string(),
        "  Deterministic - Generate deterministic password from seed".to_string(),
        "".to_string(),
        "Password Check:".to_string(),
        "  Type password and press Enter to check".to_string(),
        "  Press 'd' to toggle detailed view".to_string(),
        "".to_string(),
        "Settings:".to_string(),
        "  Use ←→ to change values".to_string(),
        "  Press Enter to save changes".to_string(),
    ];

    let content = Paragraph::new(help_text.join("\n"))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Help & Shortcuts"),
        )
        .wrap(ratatui::widgets::Wrap { trim: true });
    frame.render_widget(content, chunks[1]);

    let footer = Paragraph::new("Press Esc or q to go back")
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);
    frame.render_widget(footer, chunks[2]);
}
