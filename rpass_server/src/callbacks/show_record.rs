use super::{storage, AsyncStorage, Session, ArgIter};

/// Shows record for resource from `arg_iter` for user `session.username`
/// 
/// # Errors
/// 
/// * `UnacceptableRequestAtThisState` - if not `session.is_authorized`
/// * `EmptyResourceName` - if resource name wasn't provided
/// * `StorageError` - if can't retrieve record cause of some error in
/// `storage`
pub fn show_record(storage: AsyncStorage, session: &Session, arg_iter: ArgIter)
        -> Result<String, ShowRecordError> {
    if !session.is_authorized {
        return Err(ShowRecordError::UnacceptableRequestAtThisState);
    }

    let resource = arg_iter.next().ok_or(ShowRecordError::EmptyResourceName)?;
    let storage_read = storage.read().unwrap();
    let record = storage_read.get_record(&session.username, &resource)?;
    Ok(record.to_string())
}

#[derive(thiserror::Error, Debug)]
pub enum ShowRecordError {
    #[error("unacceptable request at this state")]
    UnacceptableRequestAtThisState,

    #[error("empty resource name")]
    EmptyResourceName,

    #[error("storage error: {0}")]
    StorageError(#[from] storage::Error)
}
