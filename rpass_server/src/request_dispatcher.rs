use std::collections::HashMap;

pub type ArgIter<'a, 'b> =&'a mut dyn Iterator<Item=&'b str>;

type Callback = dyn Fn(ArgIter) -> String + Send + Sync;

#[derive(Default)]
pub struct RequestDispatcher {
    command_to_callback: HashMap<String, Box<Callback>>
}

impl RequestDispatcher {
    pub fn add_callback<C>(&mut self, command: String, callback: C)
        where C: Fn(ArgIter) -> String + Send + Sync + 'static {
        self.command_to_callback.insert(command, Box::new(callback));
    }

    pub fn dispatch(&self, request: &str) -> Option<String> {
        let mut iter = request.split_whitespace();
        let command = match iter.next() {
            Some(cmd) => cmd,
            None => return None
        };

        match self.command_to_callback.get(command) {
            Some(callback) => Some(callback(&mut iter)),
            None => None
        }
    }
}
