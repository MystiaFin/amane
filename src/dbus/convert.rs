use std::collections::BTreeMap;

use zbus::Message;
use zbus::zvariant::{ObjectPath, Structure, StructureBuilder, Value as Variant};

use super::{Argument, Value};

pub fn value(variant: &Variant) -> Value {
    match variant {
        Variant::Bool(value) => Value::Bool(*value),

        Variant::U8(number) => Value::Number(f64::from(*number)),
        Variant::I16(number) => Value::Number(f64::from(*number)),
        Variant::U16(number) => Value::Number(f64::from(*number)),
        Variant::I32(number) => Value::Number(f64::from(*number)),
        Variant::U32(number) => Value::Number(f64::from(*number)),
        Variant::I64(number) => Value::Number(*number as f64),
        Variant::U64(number) => Value::Number(*number as f64),
        Variant::F64(number) => Value::Number(*number),

        Variant::Str(text) => Value::Text(text.to_string()),
        Variant::Signature(signature) => Value::Text(signature.to_string()),
        Variant::ObjectPath(path) => Value::Text(path.to_string()),

        // a variant only wraps another value, amane has no use for the wrapper
        Variant::Value(inner) => value(inner),

        Variant::Array(array) => Value::List(values(array.inner())),
        Variant::Structure(structure) => Value::List(values(structure.fields())),

        Variant::Dict(dict) => {
            let mut map = BTreeMap::new();

            for (key, entry) in dict.iter() {
                map.insert(key_text(key), value(entry));
            }

            Value::Map(map)
        }

        _ => Value::Nothing,
    }
}

fn values(variants: &[Variant]) -> Vec<Value> {
    variants.iter().map(value).collect()
}

// dictionary keys are almost always strings, numbers are the only other kind in use
fn key_text(key: &Variant) -> String {
    match value(key) {
        Value::Text(text) => text,
        Value::Number(number) => number.to_string(),
        _ => String::new(),
    }
}

pub fn arguments(message: &Message) -> Vec<Value> {
    let body = message.body();

    if body.is_empty() {
        return Vec::new();
    }

    // a structure reads any body, one field per argument
    let Ok(structure) = body.deserialize::<Structure>() else {
        return Vec::new();
    };

    values(structure.fields())
}

pub fn body(arguments: &[Argument]) -> Structure<'static> {
    let mut builder = StructureBuilder::new();

    for argument in arguments {
        builder = builder.append_field(variant(argument));
    }

    builder.build().expect("failed to build d-bus arguments")
}

fn variant(argument: &Argument) -> Variant<'static> {
    match argument {
        Argument::Bool(value) => Variant::Bool(*value),
        Argument::Int(number) => Variant::I32(*number),
        Argument::Unsigned(number) => Variant::U32(*number),
        Argument::Long(number) => Variant::I64(*number),
        Argument::Float(number) => Variant::F64(*number),
        Argument::Text(text) => Variant::from(text.clone()),
        Argument::Path(path) => object_path(path),
    }
}

fn object_path(path: &str) -> Variant<'static> {
    let path = ObjectPath::try_from(String::from(path))
        .expect("failed to use object path: it must look like /org/example");

    Variant::ObjectPath(path)
}
