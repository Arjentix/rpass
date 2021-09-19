use std::io;

#[derive(thiserror::Error, Debug)]
pub enum LoginError {
    #[error("empty username")]
    EmptyUsername,

    #[error("user doesn't exists")]
    NoSuchUser(#[from] io::Error)
}

#[derive(thiserror::Error, Debug)]
pub enum ConfirmLoginError {
    #[error("unacceptable request at this state")]
    UnacceptableRequestAtThisState,

    #[error("empty confirmation string")]
    EmptyConfirmationString
}
