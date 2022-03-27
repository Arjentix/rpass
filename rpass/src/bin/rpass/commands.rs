use eyre::Result;
use clap::Args;
use rpass::{key::Key, record::Record};

/// Trait to identify executable commands
pub trait Execute {
    /// Execute command with `username` and `key`
    fn execute(&self, username: &str, key: &Key) -> Result<()>;
}

/// Register new user
#[derive(Debug, Args)]
pub struct Register;

impl Execute for Register {
    fn execute(&self, _username: &str, _key: &Key) -> Result<()> {
        todo!("`Register` isn't implemented yet")
    }
}

/// Add record to database
#[derive(Debug, Args)]
pub struct Add {
    /// Record name
    record: Record,
}

impl Execute for Add {
    fn execute(&self, _username: &str, _key: &Key) -> Result<()> {
        todo!("`Add` isn't implemented yet")
    }
}

/// Delete record from database
#[derive(Debug, Args)]
pub struct Delete {
    /// Name of the record to delete
    record_name: String,
}

impl Execute for Delete {
    fn execute(&self, _username: &str, _key: &Key) -> Result<()> {
        todo!("`Delete` isn't implemented yet")
    }
}


/// Delete user from database
#[derive(Debug, Args)]
pub struct DeleteAccount;

impl Execute for DeleteAccount {
    fn execute(&self, _username: &str, _key: &Key) -> Result<()> {
        todo!("`DeleteAccount` isn't implemented yet")
    }
}

/// Delete record info
#[derive(Debug, Args)]
pub struct Get {
    /// Name of the record to get
    record_name: String,
}

impl Execute for Get {
    fn execute(&self, _username: &str, _key: &Key) -> Result<()> {
        todo!("`Get` isn't implemented yet")
    }
}

/// List all user records
#[derive(Debug, Args)]
pub struct Ls;

impl Execute for Ls {
    fn execute(&self, _username: &str, _key: &Key) -> Result<()> {
        todo!("`Ls` isn't implemented yet")
    }
}
