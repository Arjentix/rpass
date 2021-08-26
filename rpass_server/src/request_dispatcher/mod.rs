use std::collections::HashMap;

pub trait Handler : Send + Sync {
    fn handle(&mut self, args : &mut dyn Iterator<Item=&str>) -> String;
}

#[derive(Default)]
pub struct RequestDispatcher {
    command_to_handler: HashMap<String, Box<dyn Handler>>
}

impl RequestDispatcher {
    pub fn add_handler(&mut self, command: String, handler: Box<dyn Handler>) {
        self.command_to_handler.insert(command, handler);
    }

    pub fn dispatch(&mut self, request: &String) -> Option<String> {
        let mut iter = request.split_whitespace();
        let command = match iter.next() {
            Some(cmd) => cmd,
            None => return None
        };

        match self.command_to_handler.get_mut(command) {
            Some(ref mut handler) => Some(handler.handle(&mut iter)),
            None => None
        }
    }
}
