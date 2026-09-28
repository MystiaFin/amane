use zbus::fdo::{RequestNameFlags, RequestNameReply};

use super::Bus;

impl Bus {
    // false when another program already owns the name, amane never takes it from them
    pub fn own(&self, name: &str) -> bool {
        let flags = RequestNameFlags::DoNotQueue.into();

        let reply = self.connection.request_name_with_flags(name, flags);

        matches!(
            reply,
            Ok(RequestNameReply::PrimaryOwner | RequestNameReply::AlreadyOwner)
        )
    }
}
