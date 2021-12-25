pub use authorized::Authorized;
pub use unauthorized::Unauthorized;

mod authorized;
mod connector;
mod unauthorized;

#[mockall_double::double]
use connector::Connector;

use super::{error::*, record::Record, Result};
