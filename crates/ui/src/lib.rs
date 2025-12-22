use crate::app::{App, BitcoinType, Chain};
use crate::view::ui;
use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
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

#[allow(clippy::too_many_arguments)]
pub fn run_tui<F>(
    attempts: Arc<AtomicU64>,
    result_rx: mpsc::Receiver<(String, String)>,
    prefix: String,
    suffix: String,
    case_sensitive: bool,
    start_immediately: bool,
    initial_chain: Chain,
    initial_network: crate::app::Network,
    initial_btc_type: BitcoinType,
    on_search_start: F,
) -> Result<Option<(String, String)>>
where
    F: Fn(String, String, bool, Chain, crate::app::Network, BitcoinType) + Send + 'static,
{
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(
        attempts,
        prefix,
        suffix,
        case_sensitive,
        start_immediately,
        initial_chain,
        initial_network,
        initial_btc_type,
    );

    if start_immediately {
        on_search_start(
            app.prefix.clone(),
            app.suffix.clone(),
            app.case_sensitive,
            app.chain,
            app.network,
            app.btc_type,
        );
    }
    let tick_rate = Duration::from_millis(100);
    let mut last_tick = Instant::now();
    use std::time::Instant;

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

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
                            KeyCode::Backspace => app.delete_char(),
                            KeyCode::Tab | KeyCode::Down => app.next_focus(),
                            KeyCode::BackTab | KeyCode::Up => app.previous_focus(),
                            KeyCode::Right | KeyCode::Left | KeyCode::Char(' ')
                                if app.input_focus_index < 3 || app.input_focus_index == 5 =>
                            {
                                // Toggle for Chain(0), Network(1), Type(2), Case(5)
                                app.toggle_selection();
                            }
                            KeyCode::Enter => {
                                // Allow Ctrl+Enter to start from anywhere, or regular Enter on Start Button (6)
                                let is_ctrl = key.modifiers.contains(KeyModifiers::CONTROL)
                                    || key.modifiers.contains(KeyModifiers::SUPER);
                                if app.input_focus_index == 6 || is_ctrl {
                                    // Start button index is 6
                                    app.state = crate::app::AppState::Searching;
                                    app.start_time = Some(Instant::now());
                                    on_search_start(
                                        app.prefix.clone(),
                                        app.suffix.clone(),
                                        app.case_sensitive,
                                        app.chain,
                                        app.network,
                                        app.btc_type,
                                    );
                                } else if app.input_focus_index < 3 || app.input_focus_index == 5 {
                                    app.toggle_selection();
                                } else {
                                    app.next_focus();
                                }
                            }
                            KeyCode::Esc => app.should_quit = true,
                            KeyCode::Char(c) => app.enter_char(c),
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
