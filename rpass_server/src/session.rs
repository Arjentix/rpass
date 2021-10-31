use super::storage::UserStorage;
use std::sync::{Arc, RwLock};

/// Enum representing user session
pub enum Session {
    /// Initial state of every session
    Unauthorized(Unauthorized),

    /// Authorized session
    Authorized(Authorized),

    /// Session, that still exists, but has been ended
    Ended
}

#[derive(Default)]
pub struct Unauthorized {
    pub login_confirmation: String,
}

pub struct Authorized {
    pub username: String,
    pub user_storage: Arc<RwLock<UserStorage>>
}

impl Session {
    /// Checks if session is unauthorized
    pub fn is_unauthorized(&self) -> bool {
        match self {
            Unauthorized => true,
            _ => false
        }
    }

    /// Checks if session is authorized
    pub fn is_authorized(&self) -> bool {
        match self {
            Authorized => true,
            _ => false
        }
    }

    /// Checks if session is ended
    pub fn is_ended(&self) -> bool {
        match self {
            Ended => true,
            _ => false
        }
    }
}

impl Default for Session {
    fn default() -> Self {
        Session::Unauthorized(Unauthorized::default())
    }
}
