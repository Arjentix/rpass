use std::borrow::Cow;
use std::collections::HashMap;

use crate::callbacks;
use crate::session::Session;
use regex::Regex;

pub type ArgIter<'a> = &'a mut dyn Iterator<Item = String>;
pub type Result<T> = std::result::Result<T, Error>;

type Callback = dyn Fn(&mut Session, ArgIter) -> callbacks::Result<String> + Send + Sync;

mod error {
    use super::callbacks;
    use super::Cow;

    #[derive(thiserror::Error, Debug)]
    pub enum Error {
        #[error("command wasn't provided")]
        NoCommandProvided,

        #[error("undefined command `{0}`")]
        UndefinedCommand(Cow<'static, str>),

        #[error("callback error: {0}")]
        Callback(#[from] callbacks::Error),
    }
}

pub use error::Error;

lazy_static! {
    static ref ARGUMENTS_REGEX: Regex = Regex::new(r#"(?s)([^\s"]+|(?:".*?"))\s?+"#).unwrap();
}

/// Dispatches requests to registered callbacks and returns response from them
#[derive(Default)]
pub struct RequestDispatcher {
    command_to_callback: HashMap<Cow<'static, str>, Box<Callback>>,
}

impl RequestDispatcher {
    /// Creates new `RequestDispatcher`
    pub fn new() -> Self {
        Self::default()
    }

    /// Add new `callback` that will be invoked when request with `command` will be received
    ///
    /// Allows multiple adding with chaining
    pub fn add_callback<C>(&mut self, command: Cow<'static, str>, callback: C) -> &mut Self
    where
        C: Fn(&mut Session, ArgIter) -> callbacks::Result<String> + Send + Sync + 'static,
    {
        self.command_to_callback.insert(command, Box::new(callback));
        self
    }

    /// Dispatches `request` to the associated callback and return response from it
    ///
    /// # Errors
    ///
    /// * `DispatchingError::NoCommandProvided` - if `request` doesn't contains command
    /// * `DispatchingError::UndefinedCommand` - if there isn't any callback for this command
    pub fn dispatch(&self, session: &mut Session, request: &str) -> Result<String> {
        let mut iter = ARGUMENTS_REGEX
            .captures_iter(request)
            .map(|x| strip_quotes(&x[1]).to_owned());
        let command = match iter.next() {
            Some(cmd) => Cow::from(cmd),
            None => return Err(Error::NoCommandProvided),
        };

        match self.command_to_callback.get(&command) {
            Some(callback) => callback(session, &mut iter).map_err(|err| err.into()),
            None => Err(Error::UndefinedCommand(command)),
        }
    }
}

/// Strips quotes `"` from start and end of `s`.
/// Deletes only one symbol from start and end if is is equal to `"`
fn strip_quotes(s: &str) -> &str {
    if s.starts_with('\"') && s.ends_with('\"') {
        return s.strip_prefix('\"').unwrap().strip_suffix('\"').unwrap();
    }

    s
}
