use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Default)]
pub enum Value {
    // what a failed call or a missing key gives back
    #[default]
    Nothing,

    Bool(bool),

    // every d-bus number kind, so callers never pick between i32, u32 and the rest
    Number(f64),

    // strings, and object paths like /org/freedesktop/UPower
    Text(String),

    // arrays, and the fields of a struct
    List(Vec<Value>),

    Map(BTreeMap<String, Value>),
}

static NOTHING: Value = Value::Nothing;

/*
 * each getter gives an empty answer for the wrong kind,
 * so a service that is not running just shows nothing
 */
impl Value {
    pub fn bool(&self) -> bool {
        let Value::Bool(value) = self else {
            return false;
        };

        *value
    }

    pub fn number(&self) -> f64 {
        let Value::Number(number) = self else {
            return 0.0;
        };

        *number
    }

    pub fn text(&self) -> &str {
        let Value::Text(text) = self else {
            return "";
        };

        text
    }

    pub fn list(&self) -> &[Value] {
        let Value::List(list) = self else {
            return &[];
        };

        list
    }

    pub fn get(&self, key: &str) -> &Value {
        let Value::Map(map) = self else {
            return &NOTHING;
        };

        map.get(key).unwrap_or(&NOTHING)
    }
}
