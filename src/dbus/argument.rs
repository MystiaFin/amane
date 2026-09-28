use std::collections::BTreeMap;

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
