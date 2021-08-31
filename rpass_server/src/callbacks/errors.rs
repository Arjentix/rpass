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
    AlreadyExists (#[from] std::io::Error)
}
