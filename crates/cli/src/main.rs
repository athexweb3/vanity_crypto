use clap::Parser;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use std::{sync::mpsc, thread};
use vanity_ui::run_tui;
use vanity_wallet::EthereumVanityGenerator;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Prefix must start with this string (e.g., "0xDEAD")
    #[arg(short, long, default_value = "")]
    prefix: String,

    /// Suffix must end with this string (e.g., "BEEF")
    #[arg(short, long, default_value = "")]
    suffix: String,

    /// Case-sensitive matching
    #[arg(long, default_value_t = false)]
    case_sensitive: bool,

    /// Disable TUI and just print result to stdout
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
        run_batch_generation(count);
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

    // Callback to start the actual generation thread
    // We clone copies of necessary arc/channels to move into the closure
    // WARNING: We must create new channels or Arc clones inside the closure
    // effectively, or the closure must move them once.
    // Actually, TUI calls this ONCE.

    // To handle re-starts or delayed starts, we need a factory.
    // But for this v1 form, we just start once.

    // We need a thread-safe way to launch.
    // Let's create a struct or just a move closure that knows how to spawn.

    // Wait, the TUI loop runs on main thread. The closure runs on main thread.
    // The closure spawns the worker thread. Perfect.

    let on_search_start = move |p_prefix: String, p_suffix: String, p_case: bool| {
        let p_prefix = if let Some(stripped) = p_prefix.strip_prefix("0x") {
            stripped.to_string()
        } else {
            p_prefix
        };

        let gen = EthereumVanityGenerator::new(&p_prefix, &p_suffix, p_case);
        let gen = Arc::new(gen);
        let my_attempts = attempts_clone.clone();
        let my_tx = tx.clone();

        thread::spawn(move || {
            let (pk, addr) = gen.search(Some(my_attempts));
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

        // Spawn manually here as we don't call run_tui
        on_search_start(prefix, args.suffix, args.case_sensitive);

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
        // We pass empty strings if not started immediately, TUI will show empty form
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

        // If start_immediately is true, we must ALSO trigger the spawn.
        // But run_tui doesn't automatically call the callback for us initially in our design.
        // Let's manually spawn if needed, OR better: let run_tui handle it?
        // Simpler: If start_immediately, we just call the closure once before TUI?
        // No, `rx` is passed to TUI.

        if start_immediately {
            on_search_start(initial_prefix.clone(), initial_suffix.clone(), initial_case);
        }

        let result = match run_tui(
            attempts,
            rx,
            initial_prefix,
            initial_suffix,
            initial_case,
            start_immediately,
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

fn run_batch_generation(count: u64) {
    let gen = EthereumVanityGenerator::new("", "", false);

    for _ in 0..count {
        // search(None) runs infinite loop until match.
        // Since prefix is empty, it matches immediately.
        let (pk, addr) = gen.search(None);

        // JSON Lines format: {"pk": "...", "addr": "..."}
        println!("{{\"pk\": \"{}\", \"addr\": \"{}\"}}", pk, addr);
    }
}
