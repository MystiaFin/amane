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

// d-bus checks each argument's exact kind, so these keep it, unlike Value
#[derive(Debug, Clone, PartialEq)]
pub enum Argument {
    Bool(bool),

    Int(i32),

    Unsigned(u32),

    Long(i64),

    Float(f64),

    Text(String),

    // an object path like /org/freedesktop/NetworkManager, which d-bus treats apart from text
    Path(String),

    // a list of strings, like the capabilities a notification server answers with
    TextList(Vec<String>),

    // raw bytes, like a wifi network's name
    Bytes(Vec<u8>),

    // named values of any kind, like the options of a wifi scan
    Map(BTreeMap<String, Argument>),

    // named groups of named values, like a networkmanager connection's settings
    Groups(BTreeMap<String, BTreeMap<String, Argument>>),

    // a value that carries its own kind, like the new value of a property
    Variant(Box<Argument>),
}

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

impl From<bool> for Argument {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<i32> for Argument {
    fn from(value: i32) -> Self {
        Self::Int(value)
    }
}

impl From<u32> for Argument {
    fn from(value: u32) -> Self {
        Self::Unsigned(value)
    }
}

impl From<i64> for Argument {
    fn from(value: i64) -> Self {
        Self::Long(value)
    }
}

impl From<f64> for Argument {
    fn from(value: f64) -> Self {
        Self::Float(value)
    }
}

impl From<&str> for Argument {
    fn from(value: &str) -> Self {
        Self::Text(String::from(value))
    }
}

impl From<String> for Argument {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

impl From<Vec<String>> for Argument {
    fn from(value: Vec<String>) -> Self {
        Self::TextList(value)
    }
}
