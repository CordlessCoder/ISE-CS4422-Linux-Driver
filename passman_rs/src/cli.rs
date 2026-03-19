use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new, empty vault file
    CreateVault {
        /// The path to the vault to create
        #[arg(short = 'v', long = "vault")]
        path: PathBuf,
    },
    /// Read the data out of a vault file
    Unlock {
        /// The path to the vault to read from
        #[arg(short = 'v', long = "vault")]
        path: PathBuf,
        /// Skip performing the authentication check - does not ensure the vault is valid and
        /// hasn't been tampered with.
        #[arg(long)]
        skip_auth: bool,
    },
    /// Write the provided data to a vault file
    Save {
        /// The path to the vault to write to
        #[arg(short = 'v', long = "vault")]
        path: PathBuf,
        /// Skip performing the authentication check - does not ensure the vault is valid and
        /// hasn't been tampered with.
        #[arg(long)]
        skip_auth: bool,
    },
    /// Display live kernel usage statistics
    Stats {
        /// Polling interval in seconds
        #[arg(short, long, default_value_t = 0.2)]
        interval: f64,
    },
}
