use zbus::Message;
use zbus::blocking::Connection;

use super::{Argument, Value, convert};

// a call another program made to amane, waiting for its answer
pub struct Method {
    pub(crate) connection: &'static Connection,

    pub(crate) message: Message,

    pub(crate) name: String,

    pub(crate) arguments: Vec<Value>,
}

impl Method {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn arguments(&self) -> &[Value] {
        &self.arguments
    }

    // a failed reply means the caller already left, so there is nobody to tell
    pub fn reply(&self, arguments: &[Argument]) {
        let header = self.message.header();

        if arguments.is_empty() {
            let _ = self.connection.reply(&header, &());

            return;
        }

        let body = convert::body(arguments);

        let _ = self.connection.reply(&header, &body);
    }
}
