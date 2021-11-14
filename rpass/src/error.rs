
use crate::{session::Unauthorized, key::Key};
use std::str::FromStr;
use std::string::FromUtf8Error;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("can't connect to the server")]
    CantConnectToTheServer(),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("login error")]
    Login(Unauthorized),

    #[error("Invalid response")]
    InvalidResponse(#[from] FromUtf8Error),

    #[error("Invalid key")]
    InvalidKey(#[from] <Key as FromStr>::Err)
}
