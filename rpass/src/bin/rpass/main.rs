use std::{
    net::{AddrParseError, IpAddr, SocketAddr},
    path::PathBuf,
    str::FromStr,
};

use eyre::Result;
use clap::{Parser, Subcommand};
use rpass::key::Key;

use commands::Execute;

mod commands;

/// CLI utility to interact with rpass-db
#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
struct Cli {
    /// rpass_db host address (<ip>[:<port>]). Default port is 3747
    #[clap(short, long, parse(try_from_str=parse_host))]
    host: SocketAddr,
    /// Username for database
    #[clap(short, long)]
    user: String,
    /// Path to the key.sec file
    #[clap(short, long, default_value = "~/.rpass/key.sec")]
    key: PathBuf,
    /// Subcommand to run. Interactive session will be started, if no command specified
    #[clap(subcommand)]
    command: Option<Command>,
}

/// Parse host address from `s` using default port if not provided any
fn parse_host(s: &str) -> Result<SocketAddr, AddrParseError> {
    const DEFAULT_PORT: u16 = 3747;

    let addr_res = SocketAddr::from_str(s);
    match addr_res {
        Ok(host) => Ok(host),
        Err(addr_err) => {
            let ip_res = IpAddr::from_str(s);
            if let Ok(ip) = ip_res {
                Ok(SocketAddr::new(ip, DEFAULT_PORT))
            } else {
                Err(addr_err)
            }
        }
    }
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Register new user
    Register(commands::Register),

    /// Add record to database
    Add(commands::Add),

    /// Delete record from database
    Delete(commands::Delete),

    /// Delete user from database
    DeleteAccount(commands::DeleteAccount),

    /// Get record info
    Get(commands::Get),

    /// List all user records
    Ls(commands::Ls),
}

impl Execute for Command {
    fn execute(&self, username: &str, key: &Key) -> Result<()> {
        match self {
            Self::Register(command) => command.execute(username, key),
            Self::Add(command) => command.execute(username, key),
            Self::Delete(command) => command.execute(username, key),
            Self::DeleteAccount(command) => command.execute(username, key),
            Self::Get(command) => command.execute(username, key),
            Self::Ls(command) => command.execute(username, key),
        }
    }
}

fn main() -> Result<()> {
    let args = Cli::parse();

    if let Some(command) = args.command {
        let key = Key::from_file(args.key)?;
        command.execute(&args.user, &key)
    } else {
        todo!("Interactive mode isn't implemented yet")
    }
}
