use super::{AsyncStorage, Session};
use super::errors::DeleteMeError;

/// Deletes current user. Takes *username* from `session` and deletes it in
/// `storage`
/// 
/// # Errors
/// 
/// * `UnableToDelete` - if for some reason user's data can't be deleted
pub fn delete_me(storage: AsyncStorage, session: &mut Session)
        -> Result<String, DeleteMeError> {
    let mut storage_write = storage.write().unwrap();
    storage_write.delete_user(&session.username)?;
    Ok("Ok".to_owned())
}
