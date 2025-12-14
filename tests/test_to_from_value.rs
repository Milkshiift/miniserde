use miniserde::{json, Deserialize, Serialize};
use miniserde::json::Value;

#[derive(Serialize)]
struct Example {
    code: u32,
    message: String,
}

#[derive(Deserialize, PartialEq, Debug)]
struct ExampleDeserialize {
    code: u32,
    message: String,
}

#[test]
fn test_to_value() {
    let example = Example {
        code: 200,
        message: "reminiscent of Serde".to_owned(),
    };

    let value = json::to_value(&example);
    
    match value {
        Value::Object(_) => {},
        _ => panic!("Expected Value::Object"),
    }
    
    let json_string = json::to_string(&value);
    assert!(json_string.contains(r#""code":200"#));
    assert!(json_string.contains(r#""message":"reminiscent of Serde""#));
}

#[test]
fn test_to_value_primitives() {
    let bool_value = json::to_value(&true);
    assert!(matches!(bool_value, Value::Bool(true)));
    
    let num_value = json::to_value(&42u32);
    assert!(matches!(num_value, Value::Number(_)));
    
    let string_value = json::to_value(&"hello");
    assert!(matches!(string_value, Value::String(s) if s == "hello"));
}

#[test]
fn test_from_value() {
    let value = Value::Object({
        let mut map = json::Object::new();
        map.insert("code".to_string(), Value::Number(json::Number::U64(200)));
        map.insert("message".to_string(), Value::String("reminiscent of Serde".to_string()));
        map
    });

    let example: ExampleDeserialize = json::from_value(value).unwrap();
    assert_eq!(example.code, 200);
    assert_eq!(example.message, "reminiscent of Serde");
}
