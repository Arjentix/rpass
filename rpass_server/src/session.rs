use super::storage::UserStorage;
use std::sync::{Arc, RwLock};

/// Struct containing user session data
#[derive(Default)]
pub struct Session {
    pub is_authorized: bool,
    pub username: String,
    pub is_ended: bool,
    pub login_confirmation: Option<String>,
    pub user_storage: Option<Arc<RwLock<UserStorage>>>
}
