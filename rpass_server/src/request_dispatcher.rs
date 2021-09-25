pub use anyhow::Error;

use std::collections::HashMap;
use crate::session::Session;
use regex::Regex;
pub type ArgIter<'a> =&'a mut dyn Iterator<Item=String>;
pub type Result<T> = std::result::Result<T, Error>;

type Callback = dyn Fn(&mut Session, ArgIter) -> Result<String> + Send + Sync;

mod errors {

#[derive(thiserror::Error, Debug)]
pub enum DispatchingError {
    #[error("command wasn't provided")]
    NoCommandProvided,

    #[error("there is no callback for command `{0}`")]
    NoCallback(String)
}

}

use errors::*;

lazy_static! {
    static ref ARGUMENTS_REGEX: Regex
            = Regex::new(r#"(?s)([^\s"]+|(?:".*?"))\s?+"#).unwrap();
}

#[derive(Default)]
pub struct RequestDispatcher {
    command_to_callback: HashMap<String, Box<Callback>>
}

impl RequestDispatcher {
    pub fn add_callback<C>(&mut self, command: String, callback: C) -> &mut Self
        where C: Fn(&mut Session, ArgIter) -> Result<String> + Send + Sync + 'static {
        self.command_to_callback.insert(command, Box::new(callback));
        self
    }

    pub fn dispatch(&self, session: &mut Session, request: &str) -> Result<String> {
        let mut iter = ARGUMENTS_REGEX.captures_iter(request)
            .map(|x| x[1].trim_matches('\"').to_owned());
        let command = match iter.next() {
            Some(cmd) => cmd,
            None => return Err(Error::from(DispatchingError::NoCommandProvided))
        };

        match self.command_to_callback.get(&command) {
            Some(callback) => callback(session, &mut iter),
            None => Err(Error::from(
                DispatchingError::NoCallback(command.to_owned())))
        }
    }
}
