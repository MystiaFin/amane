use zbus::MatchRule;
use zbus::blocking::MessageIterator;
use zbus::message::Type;

use crate::Value;

use super::{Bus, convert};

#[derive(Debug, Clone, PartialEq)]
pub struct Signal {
    // the sender's unique name like :1.42, not a well-known name like org.mpris.MediaPlayer2.mpv
    pub(crate) sender: String,

    pub(crate) path: String,

    pub(crate) arguments: Vec<Value>,
}

// empty when the bus could not be reached
pub struct Signals {
    messages: Option<MessageIterator>,
}

impl Signal {
    pub fn sender(&self) -> &str {
        &self.sender
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn arguments(&self) -> &[Value] {
        &self.arguments
    }
}

impl Bus {
    // waits for each matching signal, so call it from a service thread, not from view()
    pub fn signals(&self, interface: &str, name: &str) -> Signals {
        let rule = MatchRule::builder()
            .msg_type(Type::Signal)
            .interface(interface)
            .expect("failed to watch signals: bad interface name")
            .member(name)
            .expect("failed to watch signals: bad signal name")
            .build();

        let messages = self
            .connection
            .and_then(|connection| MessageIterator::for_match_rule(rule, connection, None).ok());

        Signals { messages }
    }
}

impl Iterator for Signals {
    type Item = Signal;

    fn next(&mut self) -> Option<Signal> {
        loop {
            // a message that fails to read is skipped, the next one may be fine
            let Ok(message) = self.messages.as_mut()?.next()? else {
                continue;
            };

            let header = message.header();

            let sender = header.sender().map(|name| name.to_string());
            let path = header.path().map(|path| path.to_string());

            let signal = Signal {
                sender: sender.unwrap_or_default(),
                path: path.unwrap_or_default(),

                arguments: convert::arguments(&message),
            };

            return Some(signal);
        }
    }
}
