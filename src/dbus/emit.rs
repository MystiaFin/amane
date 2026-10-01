use super::{Argument, Bus, convert};

impl Bus {
    // sent to everyone watching, and a failed send only means nobody hears it
    pub fn emit(&self, path: &str, interface: &str, name: &str, arguments: &[Argument]) {
        let Some(connection) = self.connection else {
            return;
        };

        if arguments.is_empty() {
            let _ = connection.emit_signal(None::<&str>, path, interface, name, &());

            return;
        }

        let body = convert::body(arguments);

        let _ = connection.emit_signal(None::<&str>, path, interface, name, &body);
    }
}
