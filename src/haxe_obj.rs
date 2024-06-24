use std::{collections::BTreeMap, sync::RwLock};

use base64::{engine::general_purpose::STANDARD, Engine as _};
use winnow::{
    ascii::{dec_int, dec_uint, float},
    combinator::alt,
    error::ContextError,
    token::take,
    Parser, Stateful,
};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Object {
    Int(i32),
    Float(f64),
    Bool(bool),
    String(String),
    List(Vec<Option<Object>>),
    Array(Vec<Option<Object>>),
    Date(String),
    StringMap(BTreeMap<String, Option<Object>>),
    IntMap(BTreeMap<i32, Option<Object>>),
    ObjectMap(BTreeMap<Object, Option<Object>>),
    Bytes(Vec<u8>),
    Exception(String),
    Struct {
        fields: BTreeMap<String, Option<Object>>,
    },
    Class {
        name: String,
        fields: BTreeMap<String, Option<Object>>,
    },
    Enum {
        name: String,
        constructor: String,
        fields: Vec<Option<Object>>,
    },
    Custom(String),
}

#[derive(Debug, Default)]
struct ParserState {
    string_cache: Vec<String>,
    object_cache: Vec<Object>,
}

// #[derive(Debug, Clone)]
// struct ParserStateRef<'s>(&'s Cell<ParserState>);

type Input<'st> = Stateful<&'st str, &'st RwLock<ParserState>>;

pub fn parse(input: &mut &str) -> Result<Option<Object>, ContextError> {
    parse_object
        .parse(Input {
            input,
            state: &Default::default(),
        })
        .map_err(|err| err.into_inner())
}

fn parse_object(data: &mut Input) -> winnow::PResult<Option<Object>> {
    Ok(Some(match data.bytes().next().unwrap() {
        b'n' => {
            data.input = &data[1..];
            return Ok(None);
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
        b'R' => Object::String(parse_string_cache_reference(data)?),
        b'r' => parse_int_cache_reference(data)?,
        b'C' => parse_custom(data)?,
        c => todo!("{}", c as char),
    }))
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
    Ok(Object::List(items))
}

fn parse_array(data: &mut Input) -> winnow::PResult<Object> {
    'a'.parse_next(data)?;
    let mut items = Vec::new();
    while data.bytes().next() != Some(b'h') {
        if data.bytes().next() == Some(b'u') {
            'u'.parse_next(data)?;
            let count: usize = dec_uint.parse_next(data)?;
            for _ in 0..count {
                items.push(None);
            }
        } else {
            let item = parse_object(data)?;
            items.push(item);
        }
    }
    'h'.parse_next(data)?;
    Ok(Object::Array(items))
}

fn parse_date(_data: &mut Input) -> winnow::PResult<Object> {
    todo!()

    // 'v'.parse_next(data)?;
    // let date_str = take_while(|c: char| c.is_digit(10) || c == '-' || c == ' ' || c == ':')
    //     .parse_next(data)?;
    // Ok(Object::Date(date_str))
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
    Ok(Object::StringMap(map))
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
    Ok(Object::IntMap(map))
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
    Ok(Object::Bytes(bytes))
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
    // println!("enter enum {enum_name}");
    let constructor = parse_string(data)?;
    ':'.parse_next(data)?;
    let mut fields = Vec::new();
    let count: usize = dec_uint.parse_next(data)?;
    for _ in 0..count {
        let field = parse_object(data)?;
        fields.push(field);
    }
    // println!("exit enum {enum_name}");

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

fn parse_custom(_data: &mut Input) -> winnow::PResult<Object> {
    todo!()
    // 'C'.parse_next(data)?;
    // let class_name = parse_string(data)?;
    // let custom_data = take_while(|c: char| c.is_alphanumeric() || c == '_').parse_next(data)?;
    // 'g'.parse_next(data)?;
    // if let Object::String(class_name_str) = class_name {
    //     Ok(Object::Custom(format!(
    //         "{}:{}",
    //         class_name_str, custom_data
    //     )))
    // } else {
    //     Err(winnow::error::Error::new("Invalid custom class name"))
    // }
}
