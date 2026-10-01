use zbus::MatchRule;
use zbus::blocking::{Connection, MessageIterator};
use zbus::message::Type;

use super::{Bus, Method, convert};

// empty when the bus could not be reached
pub struct Methods {
    connection: Option<&'static Connection>,

    messages: Option<MessageIterator>,
}

impl Bus {
    /*
     * waits for each call to this path and interface,
     * so call it from a service thread, and before own(),
     * so the first calls are not missed
     */
    pub fn methods(&self, path: &str, interface: &str) -> Methods {
        let rule = MatchRule::builder()
            .msg_type(Type::MethodCall)
            .path(path)
            .expect("failed to watch method calls: bad path")
            .interface(interface)
            .expect("failed to watch method calls: bad interface name")
            .build();

        let messages = self
            .connection
            .and_then(|connection| MessageIterator::for_match_rule(rule, connection, None).ok());

        Methods {
            connection: self.connection,
            messages,
        }
    }
}

impl Iterator for Methods {
    type Item = Method;

    fn next(&mut self) -> Option<Method> {
        loop {
            // nothing to wait on without a bus
            let (Some(connection), Some(messages)) = (self.connection, self.messages.as_mut()) else {
                return None;
            };

            // a message that fails to read is skipped, the next one may be fine
            let Ok(message) = messages.next()? else {
                continue;
            };

            let name = message.header().member().map(|name| name.to_string());

            let method = Method {
                connection,

                name: name.unwrap_or_default(),
                arguments: convert::arguments(&message),

                message,
            };

            return Some(method);
        }
    }
}
