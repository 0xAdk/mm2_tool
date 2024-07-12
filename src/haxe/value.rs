use std::{borrow::Cow, collections::BTreeMap, fmt::Debug};

use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd)]
pub enum Value<'a> {
    Null,

    Bool(bool),
    Int(i32),
    Float(OrderedFloat<f64>),

    String(Cow<'a, str>),
    Date(Cow<'a, str>),
    Bytes(Vec<u8>),

    Array(Vec<Value<'a>>),
    List(Vec<Value<'a>>),

    StringMap(BTreeMap<Cow<'a, str>, Value<'a>>),
    IntMap(BTreeMap<i32, Value<'a>>),
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
            Value::Bytes(bytes) => write!(f, "{bytes:?}"),
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
