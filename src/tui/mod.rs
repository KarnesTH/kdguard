mod app;
mod screens;
mod ui;

use ratatui::{Terminal, backend::CrosstermBackend};
use std::io::{self, stdout};

pub use app::App;
pub use ui::ui;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    ratatui::crossterm::terminal::enable_raw_mode()?;

    let mut stdout = stdout();
    ratatui::crossterm::execute!(
        stdout,
        ratatui::crossterm::terminal::EnterAlternateScreen,
        ratatui::crossterm::cursor::Hide
    )?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let result = run_app(&mut terminal, &mut app);

    ratatui::crossterm::terminal::disable_raw_mode()?;
    ratatui::crossterm::execute!(
        terminal.backend_mut(),
        ratatui::crossterm::terminal::LeaveAlternateScreen,
        ratatui::crossterm::cursor::Show
    )?;
    terminal.show_cursor()?;

    if let Err(err) = result {
        eprintln!("Error: {}", err);
    }

    Ok(())
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if ratatui::crossterm::event::poll(std::time::Duration::from_millis(16))?
            && let ratatui::crossterm::event::Event::Key(key) = ratatui::crossterm::event::read()?
            && key.kind == ratatui::crossterm::event::KeyEventKind::Press
            && app.handle_input(key.code)
        {
            break;
        }
    }

    Ok(())
}
