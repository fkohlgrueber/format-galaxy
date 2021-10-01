
use format_galaxy_core::gen_plugin;
use json_like_value::Value;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

struct Impl {}

impl format_galaxy_core::GalaxyFormat for Impl {
    fn present(bytes: &[u8]) -> Result<String, String> {
        let val = Value::deserialize(bytes).map_err(|x| x.to_string())?;
        Ok(val.pretty_print_2())
    }

    fn store(s: &str) -> Result<Vec<u8>, String> {
        let val = Value::parse_indented(s)?;
        Ok(val.serialize())
    }
}

gen_plugin!{Impl}


#[test]
fn test() {
    let values = vec!(
        Value::Null,
        Value::Bool(true),
        Value::Bool(false),
        Value::Number(std::u64::MIN),
        Value::Number(std::u64::MAX),
        Value::String("Hello World!\nöäü€❤".to_string()),
        Value::Array(vec!()),
        Value::Array(vec!(Value::String("Nested!".to_string()))),
        Value::Array(vec!(Value::Number(1000))),
        Value::Object([
            ("Foo".to_string(), Value::Null), 
            ("❤Bar❤".to_string(), Value::Number(5000)),
            ("DeepNested".to_string(), Value::Object([
                ("bool".to_string(), Value::Bool(false)),
            ].iter().cloned().collect())),
            ("DeepNested2".to_string(), Value::Array(
                vec![Value::Number(1), Value::Number(2), Value::Number(3)]
            ))
        ].iter().cloned().collect()),
    );

    // serialize / deserialize
    for val in &values {
        let bytes = val.serialize();
        let val2 = Value::deserialize(bytes.as_slice()).expect("round trip yielded err");
        assert_eq!(val, &val2);
    }

    let exp_strings = vec!(
        "null",
        "true",
        "false",
        "0",
        "18446744073709551615",
        "\"Hello World!\\nöäü€❤\"",
        "[]",
        "[]\n  \"Nested!\"",
        "[]\n  1000",
        "{}\n  \"Foo\": null\n  \"❤Bar❤\": 5000\n  \"DeepNested\": {}\n    \"bool\": false\n  \"DeepNested2\": []\n    1\n    2\n    3",
        "[]\n  null\n  true\n  false\n  {}\n    \"a\": 1\n    \"b\": 2",
    );

    // pretty_print
    for (val, exp) in values.iter().zip(exp_strings.into_iter()) {
        let s = val.pretty_print_2();
        assert_eq!(&s, exp);
    }

    // parse
    for val in values {
        let s = val.pretty_print_2();
        let val2 = Value::parse_indented(&s).expect("parsing led to error!");
        assert_eq!(val, val2);
    }
}