use clap::Parser;
use passman_rs::{
    ClosedVault, OpenVault,
    cli::{Cli, Commands},
};
use std::io::{self, BufRead, BufReader};
use zeroize::Zeroizing;

fn main() {
    let cli = Cli::parse();
    let get_password_and_stdin = || {
        let mut stdin = BufReader::new(io::stdin().lock());
        let mut password = Zeroizing::new(Vec::new());
        stdin.read_until(b'\n', &mut password).unwrap();
        (password, stdin)
    };
    match cli.command {
        Commands::CreateVault { path } => {
            let (password, _) = get_password_and_stdin();
            OpenVault::create(&path, &password).unwrap();
            println!("Empty vault at {} created", path.display());
        }
        Commands::Unlock { path, skip_auth } => {
            let (password, _) = get_password_and_stdin();
            let vault = ClosedVault::open(&path).unwrap();
            let mut vault = vault.unlock(&password).unwrap();
            if !skip_auth {
                assert!(
                    vault.authenticate(),
                    "Authentication failed! Vault file tampered with, or password incorrect"
                );
            }
            let mut reader = vault.get_reader();
            io::copy(&mut reader, &mut io::stdout().lock()).unwrap();
        }
        Commands::Save { path, skip_auth } => {
            let (password, mut stdin) = get_password_and_stdin();
            let vault = ClosedVault::open(&path).unwrap();
            let mut vault = vault.unlock(&password).unwrap();
            if !skip_auth {
                assert!(
                    vault.authenticate(),
                    "Authentication failed! Vault file tampered with, or password incorrect"
                );
            }
            let mut writer = vault.truncate_and_get_writer();
            io::copy(&mut stdin, &mut writer).unwrap();
            writer.update_header().unwrap();
        }
        Commands::GenPassword { .. } => {
            todo!("Password generation is not implemented yet")
        }
    }
}
