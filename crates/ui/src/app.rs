use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Chain {
    Ethereum,
    Bitcoin,
}

impl Chain {
    pub fn next(&self) -> Self {
        match self {
            Chain::Ethereum => Chain::Bitcoin,
            Chain::Bitcoin => Chain::Ethereum,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BitcoinType {
    Legacy,
    SegWit,
    Taproot,
}

impl BitcoinType {
    pub fn next(&self) -> Self {
        match self {
            BitcoinType::Legacy => BitcoinType::SegWit,
            BitcoinType::SegWit => BitcoinType::Taproot,
            BitcoinType::Taproot => BitcoinType::Legacy,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Network {
    Mainnet,
    Testnet,
    Regtest,
}

impl Network {
    pub fn next(&self) -> Self {
        match self {
            Network::Mainnet => Network::Testnet,
            Network::Testnet => Network::Regtest,
            Network::Regtest => Network::Mainnet,
        }
    }
}

pub enum AppState {
    Config,
    Searching,
    Finished,
}

pub struct App {
    pub state: AppState,
    pub should_quit: bool,
    pub start_time: Option<Instant>,
    pub attempts: Arc<AtomicU64>,
    pub attempts_last_tick: u64,
    pub rate_per_second: u64,
    pub found_address: Option<(String, String)>,

    // Search Config Input
    pub chain: Chain,
    pub network: Network,
    pub btc_type: BitcoinType,
    pub prefix: String,
    pub suffix: String,
    pub case_sensitive: bool,

    // Form Focus
    // 0: Chain, 1: Network, 2: BtcType, 3: Prefix, 4: Suffix, 5: Case Sensitive, 6: Start Button
    pub input_focus_index: usize,
}

impl App {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        attempts_counter: Arc<AtomicU64>,
        prefix: String,
        suffix: String,
        case_sensitive: bool,
        start_immediately: bool,
        initial_chain: Chain,
        initial_network: Network,
        initial_btc_type: BitcoinType,
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
            chain: initial_chain,
            network: initial_network,
            btc_type: initial_btc_type,
            prefix,
            suffix,
            case_sensitive,
            input_focus_index: 3, // Start focus on Prefix (3)
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
        self.input_focus_index = (self.input_focus_index + 1) % 7;

        // Logic to skip Network(1) and BtcType(2) if Chain is Ethereum
        if self.chain == Chain::Ethereum
            && (self.input_focus_index == 1 || self.input_focus_index == 2)
        {
            self.input_focus_index = 3; // Skip to Prefix
        }
    }

    pub fn previous_focus(&mut self) {
        if self.input_focus_index == 0 {
            self.input_focus_index = 6;
        } else {
            self.input_focus_index -= 1;
        }

        // Logic to skip Network(1) and BtcType(2) if Chain is Ethereum
        if self.chain == Chain::Ethereum
            && (self.input_focus_index == 1 || self.input_focus_index == 2)
        {
            self.input_focus_index = 0; // Skip back to Chain
        }
    }

    pub fn enter_char(&mut self, c: char) {
        match self.input_focus_index {
            3 => {
                // Prefix
                // Prefix: Allow 0-9, a-f, A-F, x, plus Bitcoin chars (base58, bech32 chars)
                if c.is_alphanumeric() {
                    self.prefix.push(c);
                }
            }
            4 => {
                // Suffix
                if c.is_alphanumeric() {
                    self.suffix.push(c);
                }
            }
            0 => {
                // Chain
                // Allow 'e' or 'b' to switch?
                if c.eq_ignore_ascii_case(&'e') {
                    self.chain = Chain::Ethereum;
                }
                if c.eq_ignore_ascii_case(&'b') {
                    self.chain = Chain::Bitcoin;
                }
            }
            1 => {
                // Network
                if c.eq_ignore_ascii_case(&'m') {
                    self.network = Network::Mainnet;
                }
                if c.eq_ignore_ascii_case(&'t') {
                    self.network = Network::Testnet;
                }
                if c.eq_ignore_ascii_case(&'r') {
                    self.network = Network::Regtest;
                }
            }
            2 => {
                // BtcType
                if c.eq_ignore_ascii_case(&'l') {
                    self.btc_type = BitcoinType::Legacy;
                }
                if c.eq_ignore_ascii_case(&'s') {
                    self.btc_type = BitcoinType::SegWit;
                }
                if c.eq_ignore_ascii_case(&'p') {
                    self.btc_type = BitcoinType::Taproot;
                }
            }
            5 => {
                // Case
                if c == ' ' {
                    self.case_sensitive = !self.case_sensitive;
                }
            }
            _ => {}
        }
    }

    pub fn delete_char(&mut self) {
        match self.input_focus_index {
            3 => {
                // Prefix
                self.prefix.pop();
            }
            4 => {
                // Suffix
                self.suffix.pop();
            }
            _ => {}
        }
    }

    pub fn toggle_selection(&mut self) {
        if self.input_focus_index == 0 {
            self.chain = self.chain.next();
        } else if self.input_focus_index == 1 {
            self.network = self.network.next();
        } else if self.input_focus_index == 2 {
            self.btc_type = self.btc_type.next();
        } else if self.input_focus_index == 5 {
            self.case_sensitive = !self.case_sensitive;
        }
    }
}
