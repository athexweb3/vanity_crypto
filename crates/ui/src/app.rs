use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

pub enum AppState {
    Config,
    Searching,
    Finished,
}

pub struct App {
    pub state: AppState,
    pub should_quit: bool,
    pub start_time: Option<Instant>, // Start time is now optional (starts when searching begins)
    pub attempts: Arc<AtomicU64>,
    pub attempts_last_tick: u64,
    pub rate_per_second: u64,
    pub found_address: Option<(String, String)>,

    // Search Config Input
    pub prefix: String,
    pub suffix: String,
    pub case_sensitive: bool,

    // Form Focus
    // 0: Prefix, 1: Suffix, 2: Case Sensitive, 3: Start Button
    pub input_focus_index: usize,
}

impl App {
    pub fn new(
        attempts_counter: Arc<AtomicU64>,
        prefix: String,
        suffix: String,
        case_sensitive: bool,
        start_immediately: bool,
    ) -> Self {
        let state = if start_immediately {
            AppState::Searching
        } else {
            AppState::Config
        };

        let start_time = if start_immediately {
            Some(Instant::now())
        } else {
            None
        };

        Self {
            state,
            should_quit: false,
            start_time,
            attempts: attempts_counter,
            attempts_last_tick: 0,
            rate_per_second: 0,
            found_address: None,
            prefix,
            suffix,
            case_sensitive,
            input_focus_index: 0,
        }
    }

    pub fn on_tick(&mut self) {
        if let AppState::Searching = self.state {
            let current_attempts = self.attempts.load(Ordering::Relaxed);
            if let Some(start) = self.start_time {
                let elapsed = start.elapsed().as_secs_f64();
                if elapsed > 0.0 {
                    self.rate_per_second = (current_attempts as f64 / elapsed) as u64;
                }
            }
            self.attempts_last_tick = current_attempts;
        }
    }

    // Input Handling
    pub fn next_focus(&mut self) {
        self.input_focus_index = (self.input_focus_index + 1) % 4;
    }

    pub fn previous_focus(&mut self) {
        if self.input_focus_index == 0 {
            self.input_focus_index = 3;
        } else {
            self.input_focus_index -= 1;
        }
    }

    pub fn enter_char(&mut self, c: char) {
        match self.input_focus_index {
            0 => {
                // Prefix: Allow 0-9, a-f, A-F, x
                if c.is_ascii_hexdigit() || c == 'x' {
                    self.prefix.push(c);
                }
            }
            1 => {
                if c.is_ascii_hexdigit() {
                    self.suffix.push(c);
                }
            }
            2 => {
                // Toggle with space or enter usually, but let's allow char input to toggle too just in case
            }
            _ => {}
        }
    }

    pub fn delete_char(&mut self) {
        match self.input_focus_index {
            0 => {
                self.prefix.pop();
            }
            1 => {
                self.suffix.pop();
            }
            _ => {}
        }
    }

    pub fn toggle_case(&mut self) {
        if self.input_focus_index == 2 {
            self.case_sensitive = !self.case_sensitive;
        }
    }
}
