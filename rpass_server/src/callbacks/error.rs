use super::storage;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("unacceptable request at this state")]
    UnacceptableRequestAtThisState,

    #[error("empty username")]
    EmptyUsername,

    #[error("invalid username: {0}")]
    InvalidUsername(String),

    #[error("empty key")]
    EmptyKey,

    #[error("invalid key: `{0}`")]
    InvalidKey(#[from] rpass::key::ParseKeyError),

    #[error("empty confirmation string")]
    EmptyConfirmationString,

    #[error("invalid confirmation string")]
    InvalidConfirmationString,

    #[error("empty resource name")]
    EmptyResourceName,

    #[error("empty record content")]
    EmptyRecordContent,

    #[error("invalid record format")]
    InvalidRecordFormat(#[from] storage::ParseRecordError),

    #[error("storage error: {0}")]
    StorageError(#[from] storage::Error)
}
