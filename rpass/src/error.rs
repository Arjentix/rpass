use crate::{
    key::Key,
    record::Record,
    session::{Authorized, Unauthorized},
};
use std::str::FromStr;
use std::string::FromUtf8Error;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("can't connect to the server")]
    CantConnectToTheServer,

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("invalid response encoding")]
    InvalidResponseEncoding(#[from] FromUtf8Error),

    #[error("invalid request: {mes}")]
    InvalidRequest { mes: String },

    #[error("invalid key")]
    InvalidKey(#[from] <Key as FromStr>::Err),

    #[error("invalid resource: {mes}")]
    InvalidResource { mes: String },

    #[error("can't parse record: {0}")]
    CantParseRecord(#[from] <Record as FromStr>::Err),

    #[error("server error: {mes}")]
    Server { mes: String },

    #[error("unexpected response: {response}")]
    UnexpectedResponse { response: String },
}

#[derive(thiserror::Error, Debug)]
#[error("{source}")]
pub struct LoginError {
    pub source: Error,
    pub unauthorized: Unauthorized,
}

#[derive(thiserror::Error, Debug)]
#[error("{source}")]
pub struct DeleteMeError {
    pub source: Error,
    pub authorized: Authorized,
}
