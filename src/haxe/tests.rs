use vecmap::VecMap as Map;

use super::*;

mod roundtrip {
    use super::*;
    use value::float::Float;

    fn roundtrip_helper(data: &str, expected_values: &Vec<Value>) {
        let values = from_str(data).unwrap();
        assert_eq!(
            &values, expected_values,
            "value didn't deserialize to the expected value"
        );
        let roundtriped_data = to_string(&values);
        assert_eq!(roundtriped_data, data, "value failed to roundtrip");
    }

    #[test]
    fn null() {
        roundtrip_helper("n", &vec![Value::Null]);
    }

    #[test]
    fn bool() {
        roundtrip_helper("t", &vec![Value::Bool(true)]);
        roundtrip_helper("f", &vec![Value::Bool(false)]);
    }

    #[test]
    fn int_zero() {
        roundtrip_helper("z", &vec![Value::Int(0)]);
    }

    #[test]
    fn int() {
        roundtrip_helper("i1", &vec![Value::Int(1)]);
        roundtrip_helper("i42", &vec![Value::Int(42)]);
        roundtrip_helper("i2147483647", &vec![Value::Int(i32::MAX)]);

        roundtrip_helper("i-1", &vec![Value::Int(-1)]);
        roundtrip_helper("i-42", &vec![Value::Int(-42)]);
        roundtrip_helper("i-2147483648", &vec![Value::Int(i32::MIN)]);
    }

    #[test]
    fn float_nan() {
        roundtrip_helper("k", &vec![Value::Float(Float::new(f64::NAN))]);
    }

    #[test]
    fn float_inf() {
        roundtrip_helper("p", &vec![Value::Float(Float::new(f64::INFINITY))]);
        roundtrip_helper("m", &vec![Value::Float(Float::new(-f64::INFINITY))]);
    }

    #[test]
    fn float() {
        roundtrip_helper("d1", &vec![Value::Float(Float::new(1.0))]);
        roundtrip_helper("d0.5", &vec![Value::Float(Float::new(0.5))]);
        roundtrip_helper("d0.123", &vec![Value::Float(Float::new(0.123))]);
    }

    #[test]
    fn string() {
        // non-encoded chars
        roundtrip_helper(
            "y65:-.0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ_abcdefghijklmnopqrstuvwxyz",
            &vec![Value::String(
                "-.0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ_abcdefghijklmnopqrstuvwxyz".into(),
            )],
        );

        // encoded chars (sub control codes 0x00..=0x1F && 0x7F)
        roundtrip_helper(
        "y87:%21%22%23%24%25%26%27%28%29%2A%2B%2C%2F%3A%3B%3C%3D%3E%3F%40%5B%5C%5D%5E%60%7B%7C%7D%7E",
        &vec![Value::String("!\"#$%&'()*+,/:;<=>?@[\\]^`{|}~".into())],
    );
    }

    #[test]
    fn date() {
        roundtrip_helper(
            "v2015-10-11 20:08:02",
            &vec![Value::Date("2015-10-11 20:08:02".into())],
        );

        roundtrip_helper(
            "v2016-11-30 06:32:22",
            &vec![Value::Date("2016-11-30 06:32:22".into())],
        );

        roundtrip_helper(
            "v2017-02-01 12:38:06",
            &vec![Value::Date("2017-02-01 12:38:06".into())],
        );
    }

    #[test]
    fn bytes() {
        roundtrip_helper("s4:YQ==", &vec![Value::Bytes(vec![b'a'])]);
        roundtrip_helper(
            "s8:YWJjZA==",
            &vec![Value::Bytes(vec![b'a', b'b', b'c', b'd'])],
        );
        roundtrip_helper(
            "s8:MTIzNA==",
            &vec![Value::Bytes(vec![b'1', b'2', b'3', b'4'])],
        );
    }

    #[test]
    fn array() {
        roundtrip_helper("ah", &vec![Value::Array(vec![])]);

        roundtrip_helper(
            "azi1i2i3h",
            &vec![Value::Array(vec![
                Value::Int(0),
                Value::Int(1),
                Value::Int(2),
                Value::Int(3),
            ])],
        );

        roundtrip_helper(
            "azu5zh",
            &vec![Value::Array(vec![
                Value::Int(0),
                Value::Null,
                Value::Null,
                Value::Null,
                Value::Null,
                Value::Null,
                Value::Int(0),
            ])],
        );
    }

    #[test]
    fn list() {
        roundtrip_helper("lh", &vec![Value::List(vec![])]);

        roundtrip_helper(
            "lzi1i2i3h",
            &vec![Value::List(vec![
                Value::Int(0),
                Value::Int(1),
                Value::Int(2),
                Value::Int(3),
            ])],
        );

        roundtrip_helper(
            "lznnnnnzh",
            &vec![Value::List(vec![
                Value::Int(0),
                Value::Null,
                Value::Null,
                Value::Null,
                Value::Null,
                Value::Null,
                Value::Int(0),
            ])],
        );
    }

    #[test]
    fn string_map() {
        roundtrip_helper("bh", &vec![Value::StringMap(Map::new())]);

        roundtrip_helper(
            "by1:ai12y1:bd2.71y1:cfh",
            &vec![Value::StringMap({
                let mut map = Map::new();
                map.insert("a".into(), Value::Int(12));
                map.insert("b".into(), Value::Float(Float::new(2.71)));
                map.insert("c".into(), Value::Bool(false));
                map
            })],
        );
    }

    #[test]
    fn int_map() {
        roundtrip_helper("qh", &vec![Value::IntMap(Map::new())]);

        roundtrip_helper(
            "q:1i12:2d2.71:3fh",
            &vec![Value::IntMap({
                let mut map = Map::new();
                map.insert(1_i32, Value::Int(12));
                map.insert(2_i32, Value::Float(Float::new(2.71)));
                map.insert(3_i32, Value::Bool(false));
                map
            })],
        );
    }

    #[test]
    fn object_map() {
        roundtrip_helper("Mh", &vec![Value::ObjectMap(Map::new())]);

        roundtrip_helper(
            "Moy4:namey1:agi12oR0y1:bgd2.71oR0y1:cgfh",
            &vec![Value::ObjectMap({
                fn make_struct_key(name: &str) -> Value {
                    let fields = {
                        let mut map = Map::new();
                        map.insert("name".into(), Value::String(name.into()));
                        map
                    };

                    Value::Struct { fields }
                }

                let mut map = Map::new();
                map.insert(make_struct_key("a"), Value::Int(12));
                map.insert(make_struct_key("b"), Value::Float(Float::new(2.71)));
                map.insert(make_struct_key("c"), Value::Bool(false));
                map
            })],
        );
    }

    #[test]
    fn struct_object() {
        fn make_struct<'a, const N: usize>(fields: [(&'static str, Value<'a>); N]) -> Value<'a> {
            Value::Struct {
                fields: fields
                    .into_iter()
                    .map(|(name, value)| (name.into(), value))
                    .collect(),
            }
        }

        roundtrip_helper("og", &vec![make_struct([])]);

        roundtrip_helper(
            "oy1:bny1:ang",
            &vec![make_struct([("a", Value::Null), ("b", Value::Null)])],
        );

        roundtrip_helper(
            "oy1:any1:bng",
            &vec![make_struct([("a", Value::Null), ("b", Value::Null)])],
        );

        roundtrip_helper(
            "oy4:namey4:johny3:agei28y10:occupationy5:smithg",
            &vec![make_struct([
                ("name", Value::String("john".into())),
                ("age", Value::Int(28)),
                ("occupation", Value::String("smith".into())),
            ])],
        );
    }

    #[test]
    fn class_object() {
        fn make_class<'a, const N: usize>(
            name: &'static str,
            fields: [(&'static str, Value<'a>); N],
        ) -> Value<'a> {
            Value::Class {
                name: name.into(),
                fields: fields
                    .into_iter()
                    .map(|(name, value)| (name.into(), value))
                    .collect(),
            }
        }

        roundtrip_helper("cy0:g", &vec![make_class("", [])]);
        roundtrip_helper("cy4:testg", &vec![make_class("test", [])]);

        roundtrip_helper(
            "cy3:fooy1:bny1:ang",
            &vec![make_class("foo", [("a", Value::Null), ("b", Value::Null)])],
        );

        roundtrip_helper(
            "cy3:fooy1:any1:bng",
            &vec![make_class("foo", [("a", Value::Null), ("b", Value::Null)])],
        );

        roundtrip_helper(
            "cy6:persony4:namey4:johny3:agei28y10:occupationy5:smithg",
            &vec![make_class(
                "person",
                [
                    ("name", Value::String("john".into())),
                    ("age", Value::Int(28)),
                    ("occupation", Value::String("smith".into())),
                ],
            )],
        );
    }

    #[test]
    fn enum_object() {
        fn make_enum<'a, const N: usize>(
            name: &'static str,
            constructor: &'static str,
            fields: [Value<'a>; N],
        ) -> Value<'a> {
            Value::Enum {
                name: name.into(),
                constructor: constructor.into(),
                fields: fields.into(),
            }
        }

        roundtrip_helper("wy0:R0:0", &vec![make_enum("", "", [])]);
        roundtrip_helper("wy3:fooy0::0", &vec![make_enum("foo", "", [])]);
        roundtrip_helper("wy0:y3:foo:0", &vec![make_enum("", "foo", [])]);

        roundtrip_helper(
            "wy3:fooR0:2nz",
            &vec![make_enum("foo", "foo", [Value::Null, Value::Int(0)])],
        );

        roundtrip_helper(
            "wy3:fooR0:2zn",
            &vec![make_enum("foo", "foo", [Value::Int(0), Value::Null])],
        );

        roundtrip_helper(
            "wy3:fooy3:bar:3ni10f",
            &vec![make_enum(
                "foo",
                "bar",
                [Value::Null, Value::Int(10), Value::Bool(false)],
            )],
        );
    }

    #[test]
    fn exception() {
        todo!()
    }

    #[test]
    fn custom() {
        todo!()
    }

    #[test]
    fn string_reference() {
        roundtrip_helper(
            "y0:R0",
            &vec![Value::String("".into()), Value::String("".into())],
        );

        roundtrip_helper(
            "y1:ay1:by1:cR0R2R1",
            &vec![
                Value::String("a".into()),
                Value::String("b".into()),
                Value::String("c".into()),
                Value::String("a".into()),
                Value::String("c".into()),
                Value::String("b".into()),
            ],
        );
    }
}

#[cfg(feature = "export-json")]
mod roundtrip_json {
    use super::*;
    use value::float::Float;

    fn roundtrip_json_helper(data: &str, expected_values: &Vec<Value>) {
        let mut values = from_str(data).unwrap();
        assert_eq!(
            &values, expected_values,
            "value didn't deserialize to the expected value"
        );

        values = serde_json::from_str(&serde_json::to_string(&values).unwrap()).unwrap();
        assert_eq!(
            &values, expected_values,
            "value didn't roundtrip through json"
        );
    }

    #[test]
    fn null() {
        roundtrip_json_helper("n", &vec![Value::Null]);
    }

    #[test]
    fn bool() {
        roundtrip_json_helper("t", &vec![Value::Bool(true)]);
        roundtrip_json_helper("f", &vec![Value::Bool(false)]);
    }

    #[test]
    fn int_zero() {
        roundtrip_json_helper("z", &vec![Value::Int(0)]);
    }

    #[test]
    fn int() {
        roundtrip_json_helper("i1", &vec![Value::Int(1)]);
        roundtrip_json_helper("i42", &vec![Value::Int(42)]);
        roundtrip_json_helper("i2147483647", &vec![Value::Int(i32::MAX)]);

        roundtrip_json_helper("i-1", &vec![Value::Int(-1)]);
        roundtrip_json_helper("i-42", &vec![Value::Int(-42)]);
        roundtrip_json_helper("i-2147483648", &vec![Value::Int(i32::MIN)]);
    }

    #[test]
    fn float_nan() {
        roundtrip_json_helper("k", &vec![Value::Float(Float::new(f64::NAN))]);
    }

    #[test]
    fn float_inf() {
        roundtrip_json_helper("p", &vec![Value::Float(Float::new(f64::INFINITY))]);
        roundtrip_json_helper("m", &vec![Value::Float(Float::new(-f64::INFINITY))]);
    }

    #[test]
    fn float() {
        roundtrip_json_helper("d1", &vec![Value::Float(Float::new(1.0))]);
        roundtrip_json_helper("d0.5", &vec![Value::Float(Float::new(0.5))]);
        roundtrip_json_helper("d0.123", &vec![Value::Float(Float::new(0.123))]);
    }

    #[test]
    fn string() {
        // non-encoded chars
        roundtrip_json_helper(
            "y65:-.0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ_abcdefghijklmnopqrstuvwxyz",
            &vec![Value::String(
                "-.0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ_abcdefghijklmnopqrstuvwxyz".into(),
            )],
        );

        // encoded chars (sub control codes 0x00..=0x1F && 0x7F)
        roundtrip_json_helper(
        "y87:%21%22%23%24%25%26%27%28%29%2A%2B%2C%2F%3A%3B%3C%3D%3E%3F%40%5B%5C%5D%5E%60%7B%7C%7D%7E",
        &vec![Value::String("!\"#$%&'()*+,/:;<=>?@[\\]^`{|}~".into())],
    );
    }

    #[test]
    fn date() {
        roundtrip_json_helper(
            "v2015-10-11 20:08:02",
            &vec![Value::Date("2015-10-11 20:08:02".into())],
        );

        roundtrip_json_helper(
            "v2016-11-30 06:32:22",
            &vec![Value::Date("2016-11-30 06:32:22".into())],
        );

        roundtrip_json_helper(
            "v2017-02-01 12:38:06",
            &vec![Value::Date("2017-02-01 12:38:06".into())],
        );
    }

    #[test]
    fn bytes() {
        roundtrip_json_helper("s4:YQ==", &vec![Value::Bytes(vec![b'a'])]);
        roundtrip_json_helper(
            "s8:YWJjZA==",
            &vec![Value::Bytes(vec![b'a', b'b', b'c', b'd'])],
        );
        roundtrip_json_helper(
            "s8:MTIzNA==",
            &vec![Value::Bytes(vec![b'1', b'2', b'3', b'4'])],
        );
    }

    #[test]
    fn array() {
        roundtrip_json_helper("ah", &vec![Value::Array(vec![])]);

        roundtrip_json_helper(
            "azi1i2i3h",
            &vec![Value::Array(vec![
                Value::Int(0),
                Value::Int(1),
                Value::Int(2),
                Value::Int(3),
            ])],
        );

        roundtrip_json_helper(
            "azu5zh",
            &vec![Value::Array(vec![
                Value::Int(0),
                Value::Null,
                Value::Null,
                Value::Null,
                Value::Null,
                Value::Null,
                Value::Int(0),
            ])],
        );
    }

    #[test]
    fn list() {
        roundtrip_json_helper("lh", &vec![Value::List(vec![])]);

        roundtrip_json_helper(
            "lzi1i2i3h",
            &vec![Value::List(vec![
                Value::Int(0),
                Value::Int(1),
                Value::Int(2),
                Value::Int(3),
            ])],
        );

        roundtrip_json_helper(
            "lznnnnnzh",
            &vec![Value::List(vec![
                Value::Int(0),
                Value::Null,
                Value::Null,
                Value::Null,
                Value::Null,
                Value::Null,
                Value::Int(0),
            ])],
        );
    }

    #[test]
    fn string_map() {
        roundtrip_json_helper("bh", &vec![Value::StringMap(Map::new())]);

        roundtrip_json_helper(
            "by1:ai12y1:bd2.71y1:cfh",
            &vec![Value::StringMap({
                let mut map = Map::new();
                map.insert("a".into(), Value::Int(12));
                map.insert("b".into(), Value::Float(Float::new(2.71)));
                map.insert("c".into(), Value::Bool(false));
                map
            })],
        );
    }

    #[test]
    fn int_map() {
        roundtrip_json_helper("qh", &vec![Value::IntMap(Map::new())]);

        roundtrip_json_helper(
            "q:1i12:2d2.71:3fh",
            &vec![Value::IntMap({
                let mut map = Map::new();
                map.insert(1_i32, Value::Int(12));
                map.insert(2_i32, Value::Float(Float::new(2.71)));
                map.insert(3_i32, Value::Bool(false));
                map
            })],
        );
    }

    #[test]
    fn object_map() {
        roundtrip_json_helper("Mh", &vec![Value::ObjectMap(Map::new())]);

        roundtrip_json_helper(
            "Moy4:namey1:agi12oR0y1:bgd2.71oR0y1:cgfh",
            &vec![Value::ObjectMap({
                fn make_struct_key(name: &str) -> Value {
                    let fields = {
                        let mut map = Map::new();
                        map.insert("name".into(), Value::String(name.into()));
                        map
                    };

                    Value::Struct { fields }
                }

                let mut map = Map::new();
                map.insert(make_struct_key("a"), Value::Int(12));
                map.insert(make_struct_key("b"), Value::Float(Float::new(2.71)));
                map.insert(make_struct_key("c"), Value::Bool(false));
                map
            })],
        );
    }

    #[test]
    fn struct_object() {
        fn make_struct<'a, const N: usize>(fields: [(&'static str, Value<'a>); N]) -> Value<'a> {
            Value::Struct {
                fields: fields
                    .into_iter()
                    .map(|(name, value)| (name.into(), value))
                    .collect(),
            }
        }

        roundtrip_json_helper("og", &vec![make_struct([])]);

        roundtrip_json_helper(
            "oy1:bny1:ang",
            &vec![make_struct([("a", Value::Null), ("b", Value::Null)])],
        );

        roundtrip_json_helper(
            "oy1:any1:bng",
            &vec![make_struct([("a", Value::Null), ("b", Value::Null)])],
        );

        roundtrip_json_helper(
            "oy4:namey4:johny3:agei28y10:occupationy5:smithg",
            &vec![make_struct([
                ("name", Value::String("john".into())),
                ("age", Value::Int(28)),
                ("occupation", Value::String("smith".into())),
            ])],
        );
    }

    #[test]
    fn class_object() {
        fn make_class<'a, const N: usize>(
            name: &'static str,
            fields: [(&'static str, Value<'a>); N],
        ) -> Value<'a> {
            Value::Class {
                name: name.into(),
                fields: fields
                    .into_iter()
                    .map(|(name, value)| (name.into(), value))
                    .collect(),
            }
        }

        roundtrip_json_helper("cy0:g", &vec![make_class("", [])]);
        roundtrip_json_helper("cy4:testg", &vec![make_class("test", [])]);

        roundtrip_json_helper(
            "cy3:fooy1:bny1:ang",
            &vec![make_class("foo", [("a", Value::Null), ("b", Value::Null)])],
        );

        roundtrip_json_helper(
            "cy3:fooy1:any1:bng",
            &vec![make_class("foo", [("a", Value::Null), ("b", Value::Null)])],
        );

        roundtrip_json_helper(
            "cy6:persony4:namey4:johny3:agei28y10:occupationy5:smithg",
            &vec![make_class(
                "person",
                [
                    ("name", Value::String("john".into())),
                    ("age", Value::Int(28)),
                    ("occupation", Value::String("smith".into())),
                ],
            )],
        );
    }

    #[test]
    fn enum_object() {
        fn make_enum<'a, const N: usize>(
            name: &'static str,
            constructor: &'static str,
            fields: [Value<'a>; N],
        ) -> Value<'a> {
            Value::Enum {
                name: name.into(),
                constructor: constructor.into(),
                fields: fields.into(),
            }
        }

        roundtrip_json_helper("wy0:R0:0", &vec![make_enum("", "", [])]);
        roundtrip_json_helper("wy3:fooy0::0", &vec![make_enum("foo", "", [])]);
        roundtrip_json_helper("wy0:y3:foo:0", &vec![make_enum("", "foo", [])]);

        roundtrip_json_helper(
            "wy3:fooR0:2nz",
            &vec![make_enum("foo", "foo", [Value::Null, Value::Int(0)])],
        );

        roundtrip_json_helper(
            "wy3:fooR0:2zn",
            &vec![make_enum("foo", "foo", [Value::Int(0), Value::Null])],
        );

        roundtrip_json_helper(
            "wy3:fooy3:bar:3ni10f",
            &vec![make_enum(
                "foo",
                "bar",
                [Value::Null, Value::Int(10), Value::Bool(false)],
            )],
        );
    }

    #[test]
    fn exception() {
        todo!()
    }

    #[test]
    fn custom() {
        todo!()
    }

    #[test]
    fn string_reference() {
        roundtrip_json_helper(
            "y0:R0",
            &vec![Value::String("".into()), Value::String("".into())],
        );

        roundtrip_json_helper(
            "y1:ay1:by1:cR0R2R1",
            &vec![
                Value::String("a".into()),
                Value::String("b".into()),
                Value::String("c".into()),
                Value::String("a".into()),
                Value::String("c".into()),
                Value::String("b".into()),
            ],
        );
    }
}
