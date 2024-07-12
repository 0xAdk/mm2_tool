use std::{borrow::Cow, rc::Rc};
use std::{collections::BTreeMap, sync::RwLock};

use base64::{engine::general_purpose::STANDARD, Engine as _};
use ordered_float::OrderedFloat;
use winnow::{
    ascii::{dec_int, dec_uint, float},
    combinator::{alt, peek, repeat},
    error::ContextError,
    token::{any, take},
    Parser, Stateful,
};

use super::value::Value;

#[derive(Debug, Default)]
struct ParserState<'a> {
    string_cache: Vec<Cow<'a, str>>,
    object_cache: Vec<Value<'a>>,
}

type Input<'st> = Stateful<&'st str, Rc<RwLock<ParserState<'st>>>>;

pub fn from_str<'a>(input: &mut &'a str) -> Result<Vec<Value<'a>>, ContextError> {
    repeat(0.., parse_object)
        .parse(Input {
            input,
            state: Rc::default(),
        })
        .map_err(winnow::error::ParseError::into_inner)
}

fn parse_object<'a>(data: &mut Input<'a>) -> winnow::PResult<Value<'a>> {
    Ok(match peek(any).parse_next(data)? {
        'n' => {
            data.input = &data[1..];
            return Ok(Value::Null);
        }
        'z' => {
            data.input = &data[1..];
            Value::Int(0)
        }
        'i' => parse_int(data)?,
        'd' => parse_float(data)?,
        'k' => {
            data.input = &data[1..];
            Value::Float(OrderedFloat(f64::NAN))
        }
        'm' => {
            data.input = &data[1..];
            Value::Float(OrderedFloat(f64::NEG_INFINITY))
        }
        'p' => {
            data.input = &data[1..];
            Value::Float(OrderedFloat(f64::INFINITY))
        }
        't' => {
            data.input = &data[1..];
            Value::Bool(true)
        }
        'f' => {
            data.input = &data[1..];
            Value::Bool(false)
        }
        'y' => Value::String(parse_string_literal(data)?),
        'l' => parse_list(data)?,
        'a' => parse_array(data)?,
        'v' => parse_date(data)?,
        'b' => parse_string_map(data)?,
        'q' => parse_int_map(data)?,
        'M' => parse_object_map(data)?,
        's' => parse_bytes(data)?,
        'x' => parse_exception(data)?,
        'o' => parse_struct(data)?,
        'c' => parse_class(data)?,
        'w' => parse_enum(data)?,
        'j' => todo!("https://github.com/HaxeFoundation/haxe/blob/dc1a43dc52f98b9c480f68264885c6155e570f3e/std/haxe/Unserializer.hx#L325"),
        'R' => Value::String(parse_string_cache_reference(data)?),
        'r' => parse_int_cache_reference(data)?,
        'C' => parse_custom(data)?,
        c => todo!("{c:?}"),
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

fn parse_object_map<'a>(data: &mut Input<'a>) -> winnow::PResult<Value<'a>> {
    'M'.parse_next(data)?;
    let mut map = BTreeMap::new();
    while data.bytes().next() != Some(b'h') {
        let key = parse_object(data)?;
        let value = parse_object(data)?;
        map.insert(key, value);
    }
    'h'.parse_next(data)?;
    Ok(Value::ObjectMap(map))
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
