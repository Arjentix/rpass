pub use anyhow::Error;

use std::collections::HashMap;

use crate::session::Session;

pub type ArgIter<'a, 'b> =&'a mut dyn Iterator<Item=&'b str>;
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

#[derive(Default)]
pub struct RequestDispatcher {
    command_to_callback: HashMap<String, Box<Callback>>
}

impl RequestDispatcher {
    pub fn add_callback<C>(&mut self, command: String, callback: C)
        where C: Fn(&mut Session, ArgIter) -> Result<String> + Send + Sync + 'static {
        self.command_to_callback.insert(command, Box::new(callback));
    }

    pub fn dispatch(&self, session: &mut Session, request: &str) -> Result<String> {
        let mut iter = request.split_whitespace();
        let command = match iter.next() {
            Some(cmd) => cmd,
            None => return Err(Error::from(DispatchingError::NoCommandProvided))
        };

        match self.command_to_callback.get(command) {
            Some(callback) => callback(session, &mut iter),
            None => Err(Error::from(
                DispatchingError::NoCallback(command.to_owned())))
        }
    }
}
