use super::Connector;

/// Authorized session
///
/// Represents state when session is associated with user
#[derive(Debug)]
pub struct Authorized {
    connector: Connector,
}

impl Authorized {
    /// Creates new Authorized with `connector`
    pub(super) fn new(connector: Connector) -> Self {
        Authorized { connector }
    }
}
