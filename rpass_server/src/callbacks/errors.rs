#[derive(thiserror::Error, Debug)]
pub enum ConfirmLoginError {
    #[error("unacceptable request at this state")]
    UnacceptableRequestAtThisState,

    #[error("empty confirmation string")]
    EmptyConfirmationString
}
