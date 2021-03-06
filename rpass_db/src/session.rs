use super::storage::UserStorage;
use std::sync::{Arc, RwLock};

use enum_as_inner::EnumAsInner;

/// Enum representing user session
#[derive(EnumAsInner)]
pub enum Session {
    /// Initial state of every session
    Unauthorized(Unauthorized),

    /// Authorized session
    Authorized(Authorized),
}

#[derive(Default)]
pub struct Unauthorized {
    pub username: String,
    pub login_confirmation: String,
}

pub struct Authorized {
    pub username: String,
    pub user_storage: Arc<RwLock<UserStorage>>,
}

#[allow(dead_code)]
impl Session {
    /// Creates new `Session` initialized with `Unauthorized` variant
    pub fn new() -> Self {
        Self::default()
    }

    /// Checks if session is unauthorized
    pub fn is_unauthorized(&self) -> bool {
        matches!(self, Session::Unauthorized(_))
    }

    /// Checks if session is authorized
    pub fn is_authorized(&self) -> bool {
        matches!(self, Session::Authorized(_))
    }
}

impl Unauthorized {
    /// Creates new `Unauthorized` with all fields default-initialized
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for Session {
    fn default() -> Self {
        Session::Unauthorized(Unauthorized::default())
    }
}
