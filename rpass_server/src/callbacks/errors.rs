use std::io;

#[derive(thiserror::Error, Debug)]
pub enum RegistrationError {
    #[error("empty username")]
    EmptyUsername,

    #[error("invalid username: {0}")]
    InvalidUsername(String),

    #[error("empty key")]
    EmptyKey,

    #[error("invalid key: `{0}`")]
    InvalidKey(#[from] rpass::key::ParseBigIntError),

    #[error("user already exists")]
    AlreadyExists(#[from] io::Error)
}

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

#[derive(thiserror::Error, Debug)]
pub enum DeleteMeError {
    #[error("unable to delete")]
    UnableToDelete(#[from] io::Error)
}
