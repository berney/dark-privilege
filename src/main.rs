// This is code specific to the "static" .exe
// It uses the common code in `lib.rs` that is shared with the .dll
// The functions will be inside the build .exe, rather than loading a DLL.
// The exe will still load system DLLs etc.


use std::io::{self, Read};
use clap::{CommandFactory, Parser, Subcommand};
use env_logger::Env;
//use log::{debug, error, log_enabled, info, Level};
use log::{debug, info};
// Import all public functions
use dark_privilege_lib::*;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    ///// Optional name to operate on
    //name: Option<String>,

    ///// Sets a custom config file
    //#[arg(short, long, value_name = "FILE")]
    //config: Option<PathBuf>,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    //#[arg(short, long)]
    //http: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Who Am I?
    Whoami {
    },

    /// Get Windows Version
    Winver {
    },

    /// Get system information
    Sysinfo {
    },

    /// List privileges
    Privileges {
    },

    /// All info collection (everything except `paste`)
    All {
    },

    /// Keep Awake (like `caffeinate` or PowerToys Awake)
    KeepAwake {
        /// Active - Simulates input to keep system active
        #[arg(long)]
        active: bool,
    },

    /// Clicks the mouse and types a string into the window under the cursor
    /// Useful for locked down RDP and Citrix with clipboard disabled
    Paste {
        message: Option<Vec<String>>,
    },

    /// Prints version information
    Version,
}



fn main() {
    // Defaults to `error` level
    //env_logger::init();
    // Default to `info` level if one isn't specified
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    info!("Static exe");

    let cli = Cli::parse();
    // You can see how many times a particular flag or argument occurred
    // Note, only flags can have multiple occurrences
    match cli.debug {
        0 => debug!("Debug mode is off"),
        1 => debug!("Debug mode is kind of on"),
        2 => debug!("Debug mode is on"),
        _ => debug!("Don't be crazy"),
    }

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match &cli.command {
        Some(Commands::Winver {}) => {
            winver();
        }
        Some(Commands::Sysinfo {}) => {
            sysinfo();
        }
        Some(Commands::Whoami {}) => {
            whoami();
        }
        Some(Commands::Privileges {}) => {
            privileges();
        }
        Some(Commands::All {}) => {
            winver();
            sysinfo();
            whoami();
            privileges();
        }
        Some(Commands::KeepAwake {active}) => {
            keep_awake(*active);
        }
        Some(Commands::Paste { message }) => {
            let msg = if let Some(words) = message {
                // message argument given
                words.join(" ")
            } else {
                // use stdin
                debug!("Using stdin");
                let mut input = String::new();
                io::stdin().read_to_string(&mut input).expect("Failed to read from stdin");
                if input.is_empty() {
                    debug!("No input provided via stdin");
                }

                input
            };
            debug!("msg: {}", msg);
            paste(&msg);
        }
        Some(Commands::Version) => {
            println!("{}", Cli::command().render_version());
            // The version from Cargo.toml
            let version = env!("CARGO_PKG_VERSION");
            // embedded buy `build.rs`
            let git_version = option_env!("GIT_VERSION").unwrap_or("unknown");
            info!("Version: {}, Git: {}", version, git_version);
        }
        None => {}
    }
}
