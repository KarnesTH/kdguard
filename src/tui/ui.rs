use ratatui::Frame;

use super::app::{App, CurrentScreen};
use super::screens::{
    render_check_screen, render_generator_screen, render_help_screen, render_main_screen,
    render_settings_screen,
};

pub fn ui(frame: &mut Frame, app: &mut App) {
    match app.current_screen {
        CurrentScreen::Main => render_main_screen(frame, app),
        CurrentScreen::Generator => render_generator_screen(frame, app),
        CurrentScreen::Settings => render_settings_screen(frame, app),
        CurrentScreen::Help => render_help_screen(frame, app),
        CurrentScreen::Check => render_check_screen(frame, app),
        CurrentScreen::Exit => {
            use ratatui::layout::{Alignment, Constraint, Layout};
            use ratatui::style::{Color, Modifier, Style};
            use ratatui::widgets::{Block, Borders, Paragraph};

            let chunks = Layout::default()
                .constraints([Constraint::Length(5)])
                .split(frame.area());

            let exit_text = "Are you sure you want to exit? (y/n)";
            let exit = Paragraph::new(exit_text)
                .block(Block::default().borders(Borders::ALL).title("Exit"))
                .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
                .alignment(Alignment::Center);
            frame.render_widget(exit, chunks[0]);
        }
    }
}
