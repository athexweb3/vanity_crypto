use clap::{Parser, ValueEnum};
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use std::{sync::mpsc, thread};
use vanity_core::VanityGenerator;
use vanity_ui::{
    app::{BitcoinType as UiBtcType, Chain as UiChain, Network as UiNetwork},
    run_tui,
};
use vanity_wallet::{BitcoinAddressType, BitcoinVanityGenerator, EthereumVanityGenerator};

#[derive(Debug, Clone, ValueEnum)]
enum Chain {
    Ethereum,
    Bitcoin,
}

#[derive(Debug, Clone, ValueEnum)]
enum Network {
    Mainnet,
    Testnet,
    Regtest,
}

#[derive(Debug, Clone, ValueEnum)]
enum BtcType {
    Legacy,
    Segwit,
    Taproot,
}

impl From<Chain> for UiChain {
    fn from(c: Chain) -> Self {
        match c {
            Chain::Ethereum => UiChain::Ethereum,
            Chain::Bitcoin => UiChain::Bitcoin,
        }
    }
}

impl From<Network> for UiNetwork {
    fn from(n: Network) -> Self {
        match n {
            Network::Mainnet => UiNetwork::Mainnet,
            Network::Testnet => UiNetwork::Testnet,
            Network::Regtest => UiNetwork::Regtest,
        }
    }
}

impl From<BtcType> for UiBtcType {
    fn from(t: BtcType) -> Self {
        match t {
            BtcType::Legacy => UiBtcType::Legacy,
            BtcType::Segwit => UiBtcType::SegWit,
            BtcType::Taproot => UiBtcType::Taproot,
        }
    }
}

impl From<Network> for bitcoin::Network {
    fn from(n: Network) -> Self {
        match n {
            Network::Mainnet => bitcoin::Network::Bitcoin,
            Network::Testnet => bitcoin::Network::Testnet,
            Network::Regtest => bitcoin::Network::Regtest,
        }
    }
}

impl From<BtcType> for BitcoinAddressType {
    fn from(t: BtcType) -> Self {
        match t {
            BtcType::Legacy => BitcoinAddressType::Legacy,
            BtcType::Segwit => BitcoinAddressType::SegWit,
            BtcType::Taproot => BitcoinAddressType::Taproot,
        }
    }
}

fn convert_network(n: UiNetwork) -> bitcoin::Network {
    match n {
        UiNetwork::Mainnet => bitcoin::Network::Bitcoin,
        UiNetwork::Testnet => bitcoin::Network::Testnet,
        UiNetwork::Regtest => bitcoin::Network::Regtest,
    }
}

fn convert_btc_type(t: UiBtcType) -> BitcoinAddressType {
    match t {
        UiBtcType::Legacy => BitcoinAddressType::Legacy,
        UiBtcType::SegWit => BitcoinAddressType::SegWit,
        UiBtcType::Taproot => BitcoinAddressType::Taproot,
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Blockchain to generate address for
    #[arg(long, value_enum, default_value_t = Chain::Ethereum)]
    chain: Chain,

    /// Network (mainnet, testnet, regtest)
    #[arg(long, value_enum, default_value_t = Network::Mainnet)]
    network: Network,

    /// Bitcoin address type (only used if chain is bitcoin)
    #[arg(long, value_enum, default_value_t = BtcType::Segwit)]
    btc_type: BtcType,

    /// Prefix must start with this string (e.g., "0xDEAD")
    #[arg(short, long, default_value = "")]
    prefix: String,

    /// Suffix must end with this string (e.g., "BEEF")
    #[arg(short, long, default_value = "")]
    suffix: String,

    /// Case-sensitive matching
    #[arg(long, default_value_t = false)]
    case_sensitive: bool,

    /// Print result to stdout without TUI
    #[arg(long, default_value_t = false)]
    no_tui: bool,

    /// Generate a batch of N random keys (JSON Lines format) for fuzzing
    #[arg(long)]
    generate_batch: Option<u64>,
}

fn main() {
    let args = Args::parse();

    // Check for batch generation
    if let Some(count) = args.generate_batch {
        run_batch_generation(count, &args);
        return;
    }

    // Determine start mode
    let start_immediately = !args.prefix.is_empty() || !args.suffix.is_empty();

    // Validate if provided
    let prefix = if let Some(stripped) = args.prefix.strip_prefix("0x") {
        stripped.to_string()
    } else {
        args.prefix.clone()
    };

    // Shared state
    let attempts = Arc::new(AtomicU64::new(0));
    // Channel sends (Address, PrivateKey) strings
    let (tx, rx) = mpsc::channel::<(String, String)>();
    let attempts_clone = attempts.clone();

    // Capture configuration
    let cli_chain = args.chain.clone();
    let cli_network = args.network.clone();
    let cli_btc_type = args.btc_type.clone();

    let on_search_start = move |p_prefix: String,
                                p_suffix: String,
                                p_case: bool,
                                p_chain: UiChain,
                                p_network: UiNetwork,
                                p_btc_type: UiBtcType| {
        let p_prefix = if let Some(stripped) = p_prefix.strip_prefix("0x") {
            stripped.to_string()
        } else {
            p_prefix
        };

        let my_attempts = attempts_clone.clone();
        let my_tx = tx.clone();

        thread::spawn(move || {
            let (pk, addr) = match p_chain {
                UiChain::Ethereum => {
                    let gen = EthereumVanityGenerator::new(&p_prefix, &p_suffix, p_case);
                    gen.search(Some(my_attempts))
                }
                UiChain::Bitcoin => {
                    // Map UI Network to Bitcoin Network
                    let net = convert_network(p_network);

                    // Map UI type to Wallet type
                    let t = convert_btc_type(p_btc_type);
                    let gen = BitcoinVanityGenerator::new(&p_prefix, &p_suffix, p_case, net, t);
                    gen.search(Some(my_attempts))
                }
            };

            // Send tuple (Address, PrivateKey) as strings
            let _ = my_tx.send((addr.to_string(), pk.to_string()));
        });
    };

    if args.no_tui {
        if !start_immediately {
            eprintln!("Error: --no-tui requires --prefix or --suffix.");
            std::process::exit(1);
        }
        println!(
            "Searching for pattern defined by prefix='{}', suffix='{}'...",
            prefix, args.suffix
        );

        // Convert CLI arguments to UI modules
        let ui_chain: UiChain = cli_chain.into();
        let ui_network: UiNetwork = cli_network.into();
        let ui_btc_type: UiBtcType = cli_btc_type.into();

        // Spawn search thread directly
        on_search_start(
            prefix,
            args.suffix,
            args.case_sensitive,
            ui_chain,
            ui_network,
            ui_btc_type,
        );

        // Simple loop waiting for result
        loop {
            if let Ok(res) = rx.try_recv() {
                println!("\nAddress: {}\nPrivate Key: {}", res.0, res.1);
                run_verification(&res.1);
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    } else {
        // Run TUI on main thread
        let initial_prefix = if start_immediately {
            prefix
        } else {
            String::new()
        };
        let initial_suffix = if start_immediately {
            args.suffix
        } else {
            String::new()
        };
        let initial_case = args.case_sensitive;

        // Map CLI args to UI initial state
        let initial_ui_chain: UiChain = cli_chain.into();
        let initial_ui_network: UiNetwork = cli_network.into();
        let initial_ui_btc_type: UiBtcType = cli_btc_type.into();

        if start_immediately {
            on_search_start(
                initial_prefix.clone(),
                initial_suffix.clone(),
                initial_case,
                initial_ui_chain,
                initial_ui_network,
                initial_ui_btc_type,
            );
        }

        let result = match run_tui(
            attempts,
            rx,
            initial_prefix,
            initial_suffix,
            initial_case,
            start_immediately,
            initial_ui_chain,
            initial_ui_network,
            initial_ui_btc_type,
            on_search_start,
        ) {
            Ok(res) => res,
            Err(e) => {
                eprintln!("TUI Error: {}", e);
                None
            }
        };

        if let Some((_, pk)) = result {
            run_verification(&pk);
        }
    }
}

fn run_verification(pk: &str) {
    // Check if python3 is available
    use std::process::Command;

    println!("\n[INFO] Running Independent Verification (Python)...");

    // Path relative to binary execution (usually project root in dev)
    let script_path = std::path::Path::new("tests/verify_validate/main.py");

    // Check for script exists
    if !script_path.exists() {
        println!("⚠️  Verification script 'tests/verify_validate/main.py' not found. Skipping auto-verification.");
        return;
    }

    // Check for venv python first
    let venv_python = std::path::Path::new(".venv/bin/python");
    let system_python = if cfg!(windows) { "python" } else { "python3" };

    let python_cmd = if venv_python.exists() {
        venv_python.to_str().unwrap()
    } else {
        system_python
    };

    let output = Command::new(python_cmd).arg(script_path).arg(pk).output();

    match output {
        Ok(out) => {
            if out.status.success() {
                let stdout = String::from_utf8_lossy(&out.stdout);
                println!("{}", stdout);
            } else {
                let stderr = String::from_utf8_lossy(&out.stderr);
                println!("[ERROR] Verification Failed: {}", stderr);
                println!("Ensure 'eth-account' is installed: pip install eth-account");
            }
        }
        Err(_e) => {
            println!("Error running python verification.");
        }
    }
}

fn run_batch_generation(count: u64, args: &Args) {
    // Create generator for batch processing using a trait object to avoid code duplication
    let gen: Box<dyn VanityGenerator> = match args.chain {
        Chain::Ethereum => Box::new(EthereumVanityGenerator::new("", "", false)),
        Chain::Bitcoin => {
            let net = args.network.clone().into();
            let t = args.btc_type.clone().into();
            Box::new(BitcoinVanityGenerator::new("", "", false, net, t))
        }
    };

    for _ in 0..count {
        let (pk, addr) = gen.generate();
        println!("{{\"pk\": \"{}\", \"addr\": \"{}\"}}", pk, addr);
    }
}
