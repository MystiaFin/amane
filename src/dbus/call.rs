use super::{Argument, Bus, Value, convert};

/*
 * a failed call gives Value::Nothing instead of panicking,
 * because a player or network service that is not running is normal
 */
impl Bus {
    pub fn call(
        &self,
        destination: &str,
        path: &str,
        interface: &str,
        method: &str,
        arguments: &[Argument],
    ) -> Value {
        let reply = if arguments.is_empty() {
            self.connection
                .call_method(Some(destination), path, Some(interface), method, &())
        } else {
            let body = convert::body(arguments);

            self.connection
                .call_method(Some(destination), path, Some(interface), method, &body)
        };

        let Ok(reply) = reply else {
            return Value::Nothing;
        };

        let mut values = convert::arguments(&reply);

        // most methods answer with one value, so it comes back on its own
        match values.len() {
            0 => Value::Nothing,
            1 => values.remove(0),
            _ => Value::List(values),
        }
    }

    pub fn property(&self, destination: &str, path: &str, interface: &str, name: &str) -> Value {
        let arguments = [Argument::from(interface), Argument::from(name)];

        self.call(
            destination,
            path,
            "org.freedesktop.DBus.Properties",
            "Get",
            &arguments,
        )
    }
}
