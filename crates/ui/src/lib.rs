use crate::app::App;
use crate::view::ui;
use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::sync::atomic::AtomicU64;
pub mod app;
pub mod view;

use std::sync::mpsc;
use std::sync::Arc;
use std::{io, time::Duration};

/// Runs the TUI.
/// This function blocks until the user quits or a result is found.
///
/// `result_receiver`: A way to check if the generator thread has finished.
/// In a real app, passing a receiver channel is better.
/// For this v1, checking if `receiver.try_recv()` has a value is enough.
pub fn run_tui<F>(
    attempts: Arc<AtomicU64>,
    result_rx: mpsc::Receiver<(String, String)>, // Changed to tuple
    prefix: String,
    suffix: String,
    case_sensitive: bool,
    start_immediately: bool,
    on_search_start: F,
) -> Result<Option<(String, String)>>
where
    F: Fn(String, String, bool) + Send + 'static, // Changed FnMut to Fn and added Send + 'static
{
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(attempts, prefix, suffix, case_sensitive, start_immediately);
    let tick_rate = Duration::from_millis(100);
    let mut last_tick = Instant::now();
    use std::time::Instant;

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        // Check for result if searching
        if let crate::app::AppState::Searching = app.state {
            if let Ok(res) = result_rx.try_recv() {
                app.found_address = Some(res);
                app.state = crate::app::AppState::Finished;
            }
        }

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match app.state {
                    crate::app::AppState::Config => {
                        match key.code {
                            KeyCode::Char(c) => app.enter_char(c),
                            KeyCode::Backspace => app.delete_char(),
                            KeyCode::Tab | KeyCode::Down => app.next_focus(),
                            KeyCode::BackTab | KeyCode::Up => app.previous_focus(),
                            KeyCode::Enter => {
                                if app.input_focus_index == 3 {
                                    // Start button index is 3 (Prefix=0, Suffix=1, Case=2, Start=3) - Wait, logic says focus 3 is start button
                                    // Start search!
                                    app.state = crate::app::AppState::Searching;
                                    app.start_time = Some(Instant::now());
                                    on_search_start(
                                        app.prefix.clone(),
                                        app.suffix.clone(),
                                        app.case_sensitive,
                                    );
                                } else if app.input_focus_index == 2 {
                                    app.toggle_case();
                                } else {
                                    app.next_focus();
                                }
                            }
                            KeyCode::Esc => app.should_quit = true,
                            _ => {}
                        }
                    }
                    crate::app::AppState::Searching | crate::app::AppState::Finished => {
                        if let KeyCode::Char('q') = key.code {
                            app.should_quit = true;
                        }
                    }
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = Instant::now();
        }

        if app.should_quit {
            break;
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    // Verify cleanup happened by explicitly flushing stdout
    // crossterm's disable_raw_mode should be sufficient but explicit print helps
    if let Some(res) = &app.found_address {
        println!("{}", "=".repeat(50));
        println!("SUCCESS! Result found:");
        println!("Address: {}", res.0);
        println!("Private Key: {}", res.1);
        println!("{}", "=".repeat(50));
    } else {
        println!("Vanity Crypto: aborted.");
    }

    Ok(app.found_address)
}
