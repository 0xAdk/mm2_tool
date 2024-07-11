use std::{
    borrow::Cow,
    fmt::{self, Write},
    hash::{DefaultHasher, Hash, Hasher},
};

use base64::{engine::general_purpose::STANDARD, Engine as _};

use super::value::Value;

#[derive(Debug, Clone, Default)]
struct State {
    output: String,
    string_cache: Vec<u64>,
}

impl State {
    fn new() -> Self {
        Self::default()
    }
}

pub fn to_string(values: &[Value]) -> String {
    let mut state = State::new();

    for value in values {
        serialize_value(&mut state, value).unwrap();
    }

    state.output
}

fn serialize_value<'a>(state: &mut State, value: &'a Value<'a>) -> fmt::Result {
    let output = &mut state.output;
    match value {
        Value::Null => output.write_char('n'),
        Value::Bool(true) => output.write_char('t'),
        Value::Bool(false) => output.write_char('f'),
        Value::Int(0) => output.write_char('z'),
        Value::Int(n) => output.write_fmt(format_args!("i{n}")),
        Value::Float(n) if n.is_nan() => output.write_char('k'),
        Value::Float(n) if n.is_sign_positive() && n.is_infinite() => output.write_char('p'),
        Value::Float(n) if n.is_sign_negative() && n.is_infinite() => output.write_char('m'),
        Value::Float(n) => output.write_fmt(format_args!("d{n}")),
        Value::String(s) => serialize_string(state, s),
        Value::Date(s) => serialize_date(state, s),
        Value::Bytes(bytes) => serialize_bytes(state, bytes),
        Value::Array(v) => serialize_array(state, v),
        Value::List(v) => serialize_list(state, v),
        Value::StringMap(map) => serialize_string_map(state, map),
        Value::IntMap(map) => serialize_int_map(state, map),
        Value::ObjectMap(map) => serialize_object_map(state, map),
        Value::Struct { fields } => serialize_struct(state, fields),
        Value::Class { name, fields } => serialize_class(state, name, fields),
        Value::Enum {
            name,
            constructor,
            fields,
        } => serialize_enum(state, name, constructor, fields),
        Value::Exception(_) => todo!(),
        Value::Custom {
            name,
            fields,
            values,
        } => {
            state.output.write_char('C')?;
            serialize_string(state, name)?;
            serialize_array(
                state,
                fields
                    .iter()
                    .cloned()
                    .map(Value::String)
                    .collect::<Vec<_>>()
                    .as_slice(),
            )?;
            serialize_array(state, values)?;
            state.output.write_char('h')?;
            Ok(())
        }
    }
}

fn serialize_enum(
    state: &mut State,
    name: &str,
    constructor: &str,
    fields: &[Value],
) -> Result<(), fmt::Error> {
    state.output.write_char('w')?;
    serialize_string(state, name)?;
    serialize_string(state, constructor)?;
    state.output.write_char(':')?;
    state
        .output
        .write_fmt(format_args!("{len}", len = fields.len()))?;
    for field in fields {
        serialize_value(state, field)?;
    }
    Ok(())
}

fn serialize_class(
    state: &mut State,
    name: &str,
    fields: &std::collections::BTreeMap<Cow<'_, str>, Value<'_>>,
) -> Result<(), fmt::Error> {
    state.output.write_char('c')?;
    serialize_string(state, name)?;
    for (key, value) in fields {
        serialize_string(state, key)?;
        serialize_value(state, value)?;
    }
    state.output.write_char('g')?;
    Ok(())
}

fn serialize_struct(
    _state: &mut State,
    _fields: &std::collections::BTreeMap<Cow<'_, str>, Value<'_>>,
) -> Result<(), fmt::Error> {
    todo!()
}

fn serialize_object_map(
    _state: &mut State,
    _map: &std::collections::BTreeMap<Value<'_>, Value<'_>>,
) -> Result<(), fmt::Error> {
    todo!()
}

fn serialize_int_map(
    _state: &mut State,
    _map: &std::collections::BTreeMap<i32, Value<'_>>,
) -> Result<(), fmt::Error> {
    todo!()
}

fn serialize_string_map(
    state: &mut State,
    map: &std::collections::BTreeMap<Cow<'_, str>, Value<'_>>,
) -> Result<(), fmt::Error> {
    state.output.write_char('b')?;
    for (key, value) in map {
        serialize_string(state, key)?;
        serialize_value(state, value)?;
    }
    state.output.write_char('h')?;
    Ok(())
}

fn serialize_bytes(state: &mut State, bytes: &[u8]) -> Result<(), fmt::Error> {
    let encoded_bytes = STANDARD.encode(bytes);
    state.output.write_fmt(format_args!(
        "s{len}:{encoded_bytes}",
        len = encoded_bytes.len()
    ))
}

fn serialize_string<'a>(state: &mut State, value: &'a str) -> fmt::Result {
    let output = &mut state.output;

    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    let s_hash = hasher.finish();

    let string_cache_index = state.string_cache.iter().position(|hash| *hash == s_hash);
    if let Some(n) = string_cache_index {
        output.write_fmt(format_args!("R{n}"))
    } else {
        state.string_cache.push(s_hash);
        let encoded: Cow<'a, str> =
            percent_encoding::percent_encode(value.as_bytes(), percent_encoding::NON_ALPHANUMERIC)
                .into();

        output.write_fmt(format_args!("y{len}:{encoded}", len = encoded.len()))
    }
}

fn serialize_date(state: &mut State, value: &str) -> fmt::Result {
    state.output.write_char('v')?;
    state.output.write_str(value)?;
    Ok(())
}

fn serialize_list(state: &mut State, values: &[Value]) -> fmt::Result {
    state.output.write_char('l')?;
    for value in values {
        serialize_value(state, value)?;
    }
    state.output.write_char('h')?;

    Ok(())
}

fn serialize_array(state: &mut State, values: &[Value]) -> fmt::Result {
    state.output.write_char('a')?;
    for value in values {
        serialize_value(state, value)?;
    }
    state.output.write_char('h')?;

    Ok(())
}
