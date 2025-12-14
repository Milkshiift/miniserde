use miniserde::{json, Serialize};
use miniserde::json::Value;

#[derive(Serialize)]
struct Example {
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
    
    // Verify it's the correct type
    match value {
        Value::Object(_) => {},
        _ => panic!("Expected Value::Object"),
    }
    
    // Verify we can serialize it back
    let json_string = json::to_string(&value);
    assert!(json_string.contains(r#""code":200"#));
    assert!(json_string.contains(r#""message":"reminiscent of Serde""#));
}

#[test]
fn test_to_value_primitives() {
    // Test with primitive types
    let bool_value = json::to_value(&true);
    assert!(matches!(bool_value, Value::Bool(true)));
    
    let num_value = json::to_value(&42u32);
    assert!(matches!(num_value, Value::Number(_)));
    
    let string_value = json::to_value(&"hello");
    assert!(matches!(string_value, Value::String(s) if s == "hello"));
}
