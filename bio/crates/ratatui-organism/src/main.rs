//! Living Ratatui Organism Matrix — terminal dashboard binary.
//!
//! Renders the whole ecosystem in real time. On a 200 ms tick it advances the
//! simulation state and redraws the frame. Ctrl+C exits.

use crossterm::event::{Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::execute;
use ratatui::{Terminal};
use ratatui::backend::CrosstermBackend;
use std::error::Error;
use std::io::stdout;
use std::time::{Duration, Instant};

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let mut state = ratatui_organism::OrganismState::default();

    // Simulate a first regeneration cycle so the user sees the organism heal.
    state.simulate_regeneration("blastema-regen");

    let mut last_tick = Instant::now();
    loop {
        if last_tick.elapsed() >= Duration::from_millis(200) {
            state.tick();
            last_tick = Instant::now();
        }

        terminal.draw(|f| ratatui_organism::draw(f, &state))?;

        if crossterm::event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = crossterm::event::read()? {
                if key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(stdout(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    println!("[organism] stopped");
    Ok(())
}
