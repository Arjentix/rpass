use std::collections::HashMap;

pub struct Request {
    pub command: String,
    pub args: String
}

pub type Response = String;

pub trait Handler : Send + Sync {
    fn handle(&mut self, args : &str) -> Response;
}

#[derive(Default)]
pub struct RequestDispatcher {
    command_to_handler: HashMap<String, Box<dyn Handler>>
}

impl RequestDispatcher {
    fn add_handler(&mut self, command: String, handler: Box<dyn Handler>) {
        self.command_to_handler.insert(command, handler);
    }

    fn dispatch(&mut self, request: &Request) -> Option<Response> {
        match self.command_to_handler.get_mut(&request.command) {
            Some(ref mut handler) => Some(handler.handle(&request.args)),
            None => None
        }
    }
}
