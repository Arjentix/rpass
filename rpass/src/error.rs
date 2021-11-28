
use crate::{session::Unauthorized, key::Key};
use std::str::FromStr;
use std::string::FromUtf8Error;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("can't connect to the server")]
    CantConnectToTheServer(),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("invalid response")]
    InvalidResponse(#[from] FromUtf8Error),

    #[error("invalid key")]
    InvalidKey(#[from] <Key as FromStr>::Err),

    #[error("invalid username or key")]
    InvalidUsernameOrKey
}

#[derive(thiserror::Error, Debug)]
#[error("{source}")]
pub struct LoginError {
    pub source: Error,
    pub unauthorized: Unauthorized
}