use super::Record;

use std::io;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("io error: {0}")]
    Io(#[from] io::Error),

    #[error("Storage path {0} is not a directory")]
    StoragePathIsNotADirectory(PathBuf),

    #[error("user {0} already exists")]
    UserAlreadyExists(String),

    #[error("user {0} doesn't exist")]
    UserDoesNotExist(String),

    #[error("record parsing error: {0}")]
    CantParseRecord(#[from] <Record as FromStr>::Err),

    #[error("can't perform action cause of others active sessions")]
    UnsupportedActionForMultiSession,
}
