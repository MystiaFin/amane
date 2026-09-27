use std::collections::HashMap;

use super::IpcCall;

type Handler = fn(&[String]) -> String;

#[derive(Default)]
pub struct Handlers {
    pub(crate) by_name: HashMap<String, Handler>,
}

impl Handlers {
    pub fn insert(&mut self, name: &str, handler: Handler) {
        self.by_name.insert(String::from(name), handler);
    }

    // the reply is whatever the handler returns, so an unknown name answers in words too
    pub fn run(&self, call: &IpcCall) -> String {
        let Some(handler) = self.by_name.get(&call.name) else {
            return format!("no handler named {}", call.name);
        };

        handler(&call.arguments)
    }
}
