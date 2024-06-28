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
pub enum Object {
    Null,

    Bool(bool),
    Int(i32),
    Float(f64),

    String(String),
    Date(String),
    Bytes(Vec<u8>),

    Array(Vec<Object>),
    List(Vec<Object>),

    StringMap(BTreeMap<String, Object>),
    IntMap(BTreeMap<i32, Object>),
    #[allow(clippy::enum_variant_names)]
    ObjectMap(BTreeMap<Object, Object>),

    Struct {
        fields: BTreeMap<String, Object>,
    },

    Class {
        name: String,
        fields: BTreeMap<String, Object>,
    },

    Enum {
        name: String,
        constructor: String,
        fields: Vec<Object>,
    },

    Exception(Box<Object>),
    Custom {
        name: String,
        fields: Vec<String>,
        values: Vec<Object>,
    },
}

impl Debug for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::Null => f.write_str("null"),
            Object::Bool(value) => write!(f, "{value:?}"),
            Object::Int(value) => write!(f, "{value:?}"),
            Object::Float(value) => write!(f, "{value:?}"),
            Object::String(value) | Object::Date(value) => write!(f, "{value:?}"),
            Object::Bytes(_) => todo!(),
            Object::Array(value) | Object::List(value) => {
                f.debug_list().entries(value.iter()).finish()
            }
            Object::Custom {
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
            Object::StringMap(value) => f.debug_map().entries(value.iter()).finish(),
            Object::IntMap(value) => f.debug_map().entries(value.iter()).finish(),
            Object::ObjectMap(value) => f.debug_map().entries(value.iter()).finish(),
            Object::Struct { fields } => {
                f.write_str("struct")?;
                let mut f = f.debug_struct("");
                for (field, value) in fields {
                    f.field(field, value);
                }
                f.finish()
            }
            Object::Class { name, fields } => {
                f.write_str("class ")?;
                let mut f = f.debug_struct(name);
                for (field, value) in fields {
                    f.field(field, value);
                }
                f.finish()
            }
            Object::Enum {
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
            Object::Exception(value) => write!(f, "{value:?}"),
        }
    }
}

#[derive(Debug, Default)]
struct ParserState {
    string_cache: Vec<String>,
    object_cache: Vec<Object>,
}

type Input<'st> = Stateful<&'st str, &'st RwLock<ParserState>>;

pub fn parse(input: &mut &str) -> Result<Object, ContextError> {
    parse_object
        .parse(Input {
            input,
            state: &RwLock::default(),
        })
        .map_err(winnow::error::ParseError::into_inner)
}

fn parse_object(data: &mut Input) -> winnow::PResult<Object> {
    Ok(match data.bytes().next().unwrap() {
        b'n' => {
            data.input = &data[1..];
            return Ok(Object::Null);
        }
        b'z' => {
            data.input = &data[1..];
            Object::Int(0)
        }
        b'i' => parse_int(data)?,
        b'd' => parse_float(data)?,
        b'k' => {
            data.input = &data[1..];
            Object::Float(f64::NAN)
        }
        b'm' => {
            data.input = &data[1..];
            Object::Float(f64::NEG_INFINITY)
        }
        b'p' => {
            data.input = &data[1..];
            Object::Float(f64::INFINITY)
        }
        b't' => {
            data.input = &data[1..];
            Object::Bool(true)
        }
        b'f' => {
            data.input = &data[1..];
            Object::Bool(false)
        }
        b'y' => Object::String(parse_string_literal(data)?),
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
        b'R' => Object::String(parse_string_cache_reference(data)?),
        b'r' => parse_int_cache_reference(data)?,
        b'C' => parse_custom(data)?,
        c => todo!("{}", c as char),
    })
}

fn parse_int(data: &mut Input) -> winnow::PResult<Object> {
    'i'.parse_next(data)?;
    Ok(Object::Int(dec_int.parse_next(data)?))
}

fn parse_float(data: &mut Input) -> winnow::PResult<Object> {
    'd'.parse_next(data)?;
    Ok(Object::Float(float.parse_next(data)?))
}

fn parse_string(data: &mut Input) -> winnow::PResult<String> {
    alt((parse_string_literal, parse_string_cache_reference)).parse_next(data)
}

fn parse_string_literal(data: &mut Input) -> winnow::PResult<String> {
    'y'.parse_next(data)?;
    let len: usize = dec_uint.parse_next(data)?;
    ':'.parse_next(data)?;
    let s = take(len).parse_next(data)?;
    let s = percent_encoding::percent_decode_str(s)
        .decode_utf8()
        .unwrap()
        .into_owned();
    data.state.write().unwrap().string_cache.push(s.clone());
    Ok(s)
}

fn parse_list(data: &mut Input) -> winnow::PResult<Object> {
    'l'.parse_next(data)?;
    let mut items = Vec::new();
    while data.bytes().next() != Some(b'h') {
        let item = parse_object(data)?;
        items.push(item);
    }
    'h'.parse_next(data)?;

    let obj = Object::List(items);
    data.state.write().unwrap().object_cache.push(obj.clone());
    Ok(obj)
}

fn parse_array(data: &mut Input) -> winnow::PResult<Object> {
    'a'.parse_next(data)?;
    let mut items = Vec::new();
    while data.bytes().next() != Some(b'h') {
        if data.bytes().next() == Some(b'u') {
            'u'.parse_next(data)?;
            let count: usize = dec_uint.parse_next(data)?;
            for _ in 0..count {
                items.push(Object::Null);
            }
        } else {
            let item = parse_object(data)?;
            items.push(item);
        }
    }
    'h'.parse_next(data)?;
    let obj = Object::Array(items);
    data.state.write().unwrap().object_cache.push(obj.clone());
    Ok(obj)
}

fn parse_date(data: &mut Input) -> winnow::PResult<Object> {
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
    Ok(Object::Date(date_str.to_string()))
}

fn parse_string_map(data: &mut Input) -> winnow::PResult<Object> {
    'b'.parse_next(data)?;
    let mut map = BTreeMap::new();
    while data.bytes().next() != Some(b'h') {
        let key = parse_string(data)?;
        let value = parse_object(data)?;
        map.insert(key, value);
    }
    'h'.parse_next(data)?;
    let obj = Object::StringMap(map);
    data.state.write().unwrap().object_cache.push(obj.clone());
    Ok(obj)
}

fn parse_int_map(data: &mut Input) -> winnow::PResult<Object> {
    'q'.parse_next(data)?;
    let mut map = BTreeMap::new();
    while data.bytes().next() != Some(b'h') {
        ':'.parse_next(data)?;
        let key = dec_int.parse_next(data)?;
        let value = parse_object(data)?;
        map.insert(key, value);
    }
    'h'.parse_next(data)?;
    let obj = Object::IntMap(map);
    data.state.write().unwrap().object_cache.push(obj.clone());
    Ok(obj)
}

fn parse_object_map(_data: &mut Input) -> winnow::PResult<Object> {
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

fn parse_bytes(data: &mut Input) -> winnow::PResult<Object> {
    's'.parse_next(data)?;
    let len: usize = dec_uint.parse_next(data)?;
    ':'.parse_next(data)?;
    let bytes = take(len).parse_next(data)?;
    let bytes = STANDARD.decode(bytes).unwrap();
    let obj = Object::Bytes(bytes);
    data.state.write().unwrap().object_cache.push(obj.clone());
    Ok(obj)
}

fn parse_exception(_data: &mut Input) -> winnow::PResult<Object> {
    todo!()
    // 'x'.parse_next(data)?;
    // let exception_str = take_while(|c: char| c.is_alphanumeric()).parse_next(data)?;
    // Ok(Object::Exception(exception_str))
}

fn parse_struct(data: &mut Input) -> winnow::PResult<Object> {
    'o'.parse_next(data)?;
    let mut fields = BTreeMap::new();
    while data.bytes().next() != Some(b'g') {
        let key = parse_string(data)?;
        let value = parse_object(data)?;
        fields.insert(key, value);
    }
    'g'.parse_next(data)?;

    let obj = Object::Struct { fields };
    data.state.write().unwrap().object_cache.push(obj.clone());
    Ok(obj)
}

fn parse_class(data: &mut Input) -> winnow::PResult<Object> {
    'c'.parse_next(data)?;
    let name = parse_string(data)?;
    let mut fields = BTreeMap::new();
    while data.bytes().next() != Some(b'g') {
        let key = parse_string(data)?;
        let value = parse_object(data)?;
        fields.insert(key, value);
    }
    'g'.parse_next(data)?;

    let obj = Object::Class { name, fields };
    data.state.write().unwrap().object_cache.push(obj.clone());
    Ok(obj)
}

fn parse_enum(data: &mut Input) -> winnow::PResult<Object> {
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

    let obj = Object::Enum {
        name,
        constructor,
        fields,
    };

    data.state.write().unwrap().object_cache.push(obj.clone());
    Ok(obj)
}

fn parse_string_cache_reference(data: &mut Input) -> winnow::PResult<String> {
    'R'.parse_next(data)?;
    let index: usize = dec_uint.parse_next(data)?;
    let string_cache = &data.state.read().unwrap().string_cache;

    // TODO: maybe the strings should be under an Rc?
    Ok(string_cache.get(index).unwrap().clone())
}

fn parse_int_cache_reference(data: &mut Input) -> winnow::PResult<Object> {
    'r'.parse_next(data)?;
    let index: usize = dec_uint.parse_next(data)?;
    let object_cache = &data.state.read().unwrap().object_cache;

    Ok(object_cache.get(index).unwrap().clone())
}

fn parse_custom(data: &mut Input) -> winnow::PResult<Object> {
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
        let Object::Array(fields) = fields else {
            return Err(winnow::error::ErrMode::Cut(ContextError::new()));
        };
        fields
            .into_iter()
            .map(|obj| {
                let Object::String(s) = obj else {
                    return Err(winnow::error::ErrMode::Cut(ContextError::new()));
                };

                winnow::PResult::Ok(s)
            })
            .collect::<winnow::PResult<Vec<String>>>()?
    };
    let values = {
        let values = parse_array.parse_next(data)?;
        let Object::Array(values) = values else {
            return Err(winnow::error::ErrMode::Cut(ContextError::new()));
        };
        values
    };
    'g'.parse_next(data)?;

    let obj = Object::Custom {
        name,
        fields,
        values,
    };
    data.state.write().unwrap().object_cache.push(obj.clone());
    Ok(obj)
}
