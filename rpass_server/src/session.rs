/// Struct containing user session data
#[derive(Default, Debug)]
pub struct Session {
    pub login_confirmation: Option<String>,
    pub is_authorized: bool
}
