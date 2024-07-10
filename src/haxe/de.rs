use std::{borrow::Cow, rc::Rc};
use std::{collections::BTreeMap, fmt::Debug, sync::RwLock};

use base64::{engine::general_purpose::STANDARD, Engine as _};
use winnow::{
    ascii::{dec_int, dec_uint, float},
    combinator::alt,
    error::ContextError,
    token::take,
    Parser, Stateful,
};

#[allow(dead_code)]
#[derive(Clone, PartialEq, PartialOrd)]
pub enum Value<'a> {
    Null,

    Bool(bool),
    Int(i32),
    Float(f64),

    String(Cow<'a, str>),
    Date(Cow<'a, str>),
    Bytes(Vec<u8>),

    Array(Vec<Value<'a>>),
    List(Vec<Value<'a>>),

    StringMap(BTreeMap<Cow<'a, str>, Value<'a>>),
    IntMap(BTreeMap<i32, Value<'a>>),
    #[allow(clippy::enum_variant_names)]
    ObjectMap(BTreeMap<Value<'a>, Value<'a>>),

    Struct {
        fields: BTreeMap<Cow<'a, str>, Value<'a>>,
    },

    Class {
        name: Cow<'a, str>,
        fields: BTreeMap<Cow<'a, str>, Value<'a>>,
    },

    Enum {
        name: Cow<'a, str>,
        constructor: Cow<'a, str>,
        fields: Vec<Value<'a>>,
    },

    Exception(Box<Value<'a>>),
    Custom {
        name: Cow<'a, str>,
        fields: Vec<Cow<'a, str>>,
        values: Vec<Value<'a>>,
    },
}

impl Debug for Value<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Null => f.write_str("null"),
            Value::Bool(value) => write!(f, "{value:?}"),
            Value::Int(value) => write!(f, "{value:?}"),
            Value::Float(value) => write!(f, "{value:?}"),
            Value::String(value) | Value::Date(value) => write!(f, "{value:?}"),
            Value::Bytes(_) => todo!(),
            Value::Array(value) | Value::List(value) => {
                f.debug_list().entries(value.iter()).finish()
            }
            Value::Custom {
                name,
                fields,
                values,
            } => {
                f.write_str("class ")?;
                let mut f = f.debug_struct(name);
                let entries = fields.iter().zip(values.iter());
                for (field, value) in entries {
                    f.field(field, value);
                }
                f.finish()
            }
            Value::StringMap(value) => f.debug_map().entries(value.iter()).finish(),
            Value::IntMap(value) => f.debug_map().entries(value.iter()).finish(),
            Value::ObjectMap(value) => f.debug_map().entries(value.iter()).finish(),
            Value::Struct { fields } => {
                f.write_str("struct")?;
                let mut f = f.debug_struct("");
                for (field, value) in fields {
                    f.field(field, value);
                }
                f.finish()
            }
            Value::Class { name, fields } => {
                f.write_str("class ")?;
                let mut f = f.debug_struct(name);
                for (field, value) in fields {
                    f.field(field, value);
                }
                f.finish()
            }
            Value::Enum {
                name,
                constructor,
                fields,
            } => {
                let mut f = f.debug_tuple(format!("{name}.{constructor}").as_str());
                for field in fields {
                    f.field(field);
                }
                f.finish()
            }
            Value::Exception(value) => write!(f, "{value:?}"),
        }
    }
}

#[derive(Debug, Default)]
struct ParserState<'a> {
    string_cache: Vec<Cow<'a, str>>,
    object_cache: Vec<Value<'a>>,
}

type Input<'st> = Stateful<&'st str, Rc<RwLock<ParserState<'st>>>>;

pub fn parse<'a>(input: &mut &'a str) -> Result<Value<'a>, ContextError> {
    parse_object
        .parse(Input {
            input,
            state: Rc::default(),
        })
        .map_err(winnow::error::ParseError::into_inner)
}

fn parse_object<'a>(data: &mut Input<'a>) -> winnow::PResult<Value<'a>> {
    Ok(match data.bytes().next().unwrap() {
        b'n' => {
            data.input = &data[1..];
            return Ok(Value::Null);
        }
        b'z' => {
            data.input = &data[1..];
            Value::Int(0)
        }
        b'i' => parse_int(data)?,
        b'd' => parse_float(data)?,
        b'k' => {
            data.input = &data[1..];
            Value::Float(f64::NAN)
        }
        b'm' => {
            data.input = &data[1..];
            Value::Float(f64::NEG_INFINITY)
        }
        b'p' => {
            data.input = &data[1..];
            Value::Float(f64::INFINITY)
        }
        b't' => {
            data.input = &data[1..];
            Value::Bool(true)
        }
        b'f' => {
            data.input = &data[1..];
            Value::Bool(false)
        }
        b'y' => Value::String(parse_string_literal(data)?),
        b'l' => parse_list(data)?,
        b'a' => parse_array(data)?,
        b'v' => parse_date(data)?,
        b'b' => parse_string_map(data)?,
        b'q' => parse_int_map(data)?,
        b'M' => parse_object_map(data)?,
        b's' => parse_bytes(data)?,
        b'x' => parse_exception(data)?,
        b'o' => parse_struct(data)?,
        b'c' => parse_class(data)?,
        b'w' => parse_enum(data)?,
        b'j' => todo!("https://github.com/HaxeFoundation/haxe/blob/dc1a43dc52f98b9c480f68264885c6155e570f3e/std/haxe/Unserializer.hx#L325"),
        b'R' => Value::String(parse_string_cache_reference(data)?),
        b'r' => parse_int_cache_reference(data)?,
        b'C' => parse_custom(data)?,
        c => todo!("{}", c as char),
    })
}

fn parse_int<'a>(data: &mut Input<'a>) -> winnow::PResult<Value<'a>> {
    'i'.parse_next(data)?;
    Ok(Value::Int(dec_int.parse_next(data)?))
}

fn parse_float<'a>(data: &mut Input<'a>) -> winnow::PResult<Value<'a>> {
    'd'.parse_next(data)?;
    Ok(Value::Float(float.parse_next(data)?))
}

fn parse_string<'a>(data: &mut Input<'a>) -> winnow::PResult<Cow<'a, str>> {
    alt((parse_string_literal, parse_string_cache_reference)).parse_next(data)
}

fn parse_string_literal<'a>(data: &mut Input<'a>) -> winnow::PResult<Cow<'a, str>> {
    'y'.parse_next(data)?;
    let len: usize = dec_uint.parse_next(data)?;
    ':'.parse_next(data)?;
    let s = take(len).parse_next(data)?;
    let s = percent_encoding::percent_decode_str(s)
        .decode_utf8()
        .unwrap();
    data.state.write().unwrap().string_cache.push(s.clone());
    Ok(s)
}

fn parse_list<'a>(data: &mut Input<'a>) -> winnow::PResult<Value<'a>> {
    'l'.parse_next(data)?;
    let mut items = Vec::new();
    while data.bytes().next() != Some(b'h') {
        let item = parse_object(data)?;
        items.push(item);
    }
    'h'.parse_next(data)?;

    let obj = Value::List(items);
    data.state.write().unwrap().object_cache.push(obj.clone());
    Ok(obj)
}

fn parse_array<'a>(data: &mut Input<'a>) -> winnow::PResult<Value<'a>> {
    'a'.parse_next(data)?;
    let mut items = Vec::new();
    while data.bytes().next() != Some(b'h') {
        if data.bytes().next() == Some(b'u') {
            'u'.parse_next(data)?;
            let count: usize = dec_uint.parse_next(data)?;
            for _ in 0..count {
                items.push(Value::Null);
            }
        } else {
            let item = parse_object(data)?;
            items.push(item);
        }
    }
    'h'.parse_next(data)?;
    let obj = Value::Array(items);
    data.state.write().unwrap().object_cache.push(obj.clone());
    Ok(obj)
}

fn parse_date<'a>(data: &mut Input<'a>) -> winnow::PResult<Value<'a>> {
    'v'.parse_next(data)?;

    // let year = dec_uint.parse_next(data)?;
    // '-'.parse_next(data)?;
    // let month = dec_uint.parse_next(data)?;
    // '-'.parse_next(data)?;
    // let day = dec_uint.parse_next(data)?;
    // ' '.parse_next(data)?;
    // let hour = dec_uint.parse_next(data)?;
    // ':'.parse_next(data)?;
    // let minute = dec_uint.parse_next(data)?;
    // ':'.parse_next(data)?;
    // let second = dec_uint.parse_next(data)?;

    let date_str = take(19_usize).parse_next(data)?;
    Ok(Value::Date(date_str.into()))
}

fn parse_string_map<'a>(data: &mut Input<'a>) -> winnow::PResult<Value<'a>> {
    'b'.parse_next(data)?;
    let mut map = BTreeMap::new();
    while data.bytes().next() != Some(b'h') {
        let key = parse_string(data)?;
        let value = parse_object(data)?;
        map.insert(key, value);
    }
    'h'.parse_next(data)?;
    let obj = Value::StringMap(map);
    data.state.write().unwrap().object_cache.push(obj.clone());
    Ok(obj)
}

fn parse_int_map<'a>(data: &mut Input<'a>) -> winnow::PResult<Value<'a>> {
    'q'.parse_next(data)?;
    let mut map = BTreeMap::new();
    while data.bytes().next() != Some(b'h') {
        ':'.parse_next(data)?;
        let key = dec_int.parse_next(data)?;
        let value = parse_object(data)?;
        map.insert(key, value);
    }
    'h'.parse_next(data)?;
    let obj = Value::IntMap(map);
    data.state.write().unwrap().object_cache.push(obj.clone());
    Ok(obj)
}

fn parse_object_map<'a>(_data: &mut Input<'a>) -> winnow::PResult<Value<'a>> {
    todo!()
    // 'M'.parse_next(data)?;
    // let mut map = BTreeMap::new();
    // while data.bytes().next() != Some(b'h') {
    //     let key = parse_object(data)?;
    //     let value = parse_object(data)?;
    //     if let Some(key) = key {
    //         map.insert(key, Some(value));
    //     }
    // }
    // 'h'.parse_next(data)?;
    // Ok(Object::ObjectMap(map))
}

fn parse_bytes<'a>(data: &mut Input<'a>) -> winnow::PResult<Value<'a>> {
    's'.parse_next(data)?;
    let len: usize = dec_uint.parse_next(data)?;
    ':'.parse_next(data)?;
    let bytes = take(len).parse_next(data)?;
    let bytes = STANDARD.decode(bytes).unwrap();
    let obj = Value::Bytes(bytes);
    data.state.write().unwrap().object_cache.push(obj.clone());
    Ok(obj)
}

fn parse_exception<'a>(_data: &mut Input<'a>) -> winnow::PResult<Value<'a>> {
    todo!()
    // 'x'.parse_next(data)?;
    // let exception_str = take_while(|c: char| c.is_alphanumeric()).parse_next(data)?;
    // Ok(Object::Exception(exception_str))
}

fn parse_struct<'a>(data: &mut Input<'a>) -> winnow::PResult<Value<'a>> {
    'o'.parse_next(data)?;
    let mut fields = BTreeMap::new();
    while data.bytes().next() != Some(b'g') {
        let key = parse_string(data)?;
        let value = parse_object(data)?;
        fields.insert(key, value);
    }
    'g'.parse_next(data)?;

    let obj = Value::Struct { fields };
    data.state.write().unwrap().object_cache.push(obj.clone());
    Ok(obj)
}

fn parse_class<'a>(data: &mut Input<'a>) -> winnow::PResult<Value<'a>> {
    'c'.parse_next(data)?;
    let name = parse_string(data)?;
    let mut fields = BTreeMap::new();
    while data.bytes().next() != Some(b'g') {
        let key = parse_string(data)?;
        let value = parse_object(data)?;
        fields.insert(key, value);
    }
    'g'.parse_next(data)?;

    let obj = Value::Class { name, fields };
    data.state.write().unwrap().object_cache.push(obj.clone());
    Ok(obj)
}

fn parse_enum<'a>(data: &mut Input<'a>) -> winnow::PResult<Value<'a>> {
    'w'.parse_next(data)?;
    let name = parse_string(data)?;
    let constructor = parse_string(data)?;
    ':'.parse_next(data)?;
    let mut fields = Vec::new();
    let count: usize = dec_uint.parse_next(data)?;
    for _ in 0..count {
        let field = parse_object(data)?;
        fields.push(field);
    }

    let obj = Value::Enum {
        name,
        constructor,
        fields,
    };

    data.state.write().unwrap().object_cache.push(obj.clone());
    Ok(obj)
}

fn parse_string_cache_reference<'a>(data: &mut Input<'a>) -> winnow::PResult<Cow<'a, str>> {
    'R'.parse_next(data)?;
    let index: usize = dec_uint.parse_next(data)?;
    let string_cache = &data.state.read().unwrap().string_cache;

    // TODO: maybe the strings should be under an Rc?
    Ok(string_cache.get(index).unwrap().clone())
}

fn parse_int_cache_reference<'a>(data: &mut Input<'a>) -> winnow::PResult<Value<'a>> {
    'r'.parse_next(data)?;
    let index: usize = dec_uint.parse_next(data)?;
    let object_cache = &data.state.read().unwrap().object_cache;

    Ok(object_cache.get(index).unwrap().clone())
}

fn parse_custom<'a>(data: &mut Input<'a>) -> winnow::PResult<Value<'a>> {
    'C'.parse_next(data)?;
    let name = parse_string.parse_next(data)?;
    // technically after the class there is arbitrary data, but from testing
    // the data I care about custom sections just contains more serialized haxe
    // data.
    //
    // That data is always two arrays one with strings that are field
    // name, and another with the same number of elements of the last array
    // with each fields value. If this ever caues a panic on deserialization
    // I'll have to rethink this xd
    let fields = {
        let fields = parse_array.parse_next(data)?;
        let Value::Array(fields) = fields else {
            return Err(winnow::error::ErrMode::Cut(ContextError::new()));
        };
        fields
            .into_iter()
            .map(|obj| {
                let Value::String(s) = obj else {
                    return Err(winnow::error::ErrMode::Cut(ContextError::new()));
                };

                winnow::PResult::Ok(s)
            })
            .collect::<winnow::PResult<Vec<_>>>()?
    };
    let values = {
        let values = parse_array.parse_next(data)?;
        let Value::Array(values) = values else {
            return Err(winnow::error::ErrMode::Cut(ContextError::new()));
        };
        values
    };
    'g'.parse_next(data)?;

    let obj = Value::Custom {
        name,
        fields,
        values,
    };
    data.state.write().unwrap().object_cache.push(obj.clone());
    Ok(obj)
}
