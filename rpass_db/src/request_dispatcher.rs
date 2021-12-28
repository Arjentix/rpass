pub use anyhow::Error;
pub use std::borrow::Cow;

use crate::session::Session;
use regex::Regex;
use std::collections::HashMap;

pub type ArgIter<'a> = &'a mut dyn Iterator<Item = String>;
pub type Result<T> = std::result::Result<T, Error>;

type Callback = dyn Fn(&mut Session, ArgIter) -> Result<String> + Send + Sync;

mod error {

    use super::Cow;

    #[derive(thiserror::Error, Debug)]
    pub enum DispatchingError {
        #[error("command wasn't provided")]
        NoCommandProvided,

        #[error("undefined command `{0}`")]
        UndefinedCommand(Cow<'static, str>),
    }
}

use error::*;

lazy_static! {
    static ref ARGUMENTS_REGEX: Regex = Regex::new(r#"(?s)([^\s"]+|(?:".*?"))\s?+"#).unwrap();
}

#[derive(Default)]
pub struct RequestDispatcher {
    command_to_callback: HashMap<Cow<'static, str>, Box<Callback>>,
}

impl RequestDispatcher {
    /// Creates new `RequestDispatcher`
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_callback<C>(&mut self, command: Cow<'static, str>, callback: C) -> &mut Self
    where
        C: Fn(&mut Session, ArgIter) -> Result<String> + Send + Sync + 'static,
    {
        self.command_to_callback.insert(command, Box::new(callback));
        self
    }

    pub fn dispatch(&self, session: &mut Session, request: &str) -> Result<String> {
        let mut iter = ARGUMENTS_REGEX
            .captures_iter(request)
            .map(|x| strip_quotes(&x[1]).to_owned());
        let command = match iter.next() {
            Some(cmd) => Cow::from(cmd),
            None => return Err(Error::from(DispatchingError::NoCommandProvided)),
        };

        match self.command_to_callback.get(&command) {
            Some(callback) => callback(session, &mut iter),
            None => Err(Error::from(DispatchingError::UndefinedCommand(command))),
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
