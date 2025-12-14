#![allow(clippy::uninlined_format_args)]

use indoc::indoc;
use miniserde::json::{self, Value};

#[test]
fn test_round_trip_deeply_nested() {
    let depth = if cfg!(miri) { 40 } else { 100_000 };

    let mut j = String::new();
    for _ in 0..depth {
        j.push_str("{\"x\":[");
    }
    for _ in 0..depth {
        j.push_str("]}");
    }

    let value: Value = json::from_str(&j).unwrap();
    let j2 = json::to_string(&value);
    assert_eq!(j, j2);
}

#[test]
fn test_debug() {
    let j = r#"
        {
            "Null": null,
            "Bool": true,
            "Number": 1,
            "String": "...",
            "Array": [true],
            "EmptyArray": [],
            "EmptyObject": {}
        }
    "#;

    let value: Value = json::from_str(j).unwrap();
    let debug = format!("{:#?}", value);

    let expected = indoc! {r#"
        Object {
            "Array": Array [
                Bool(true),
            ],
            "Bool": Bool(true),
            "EmptyArray": Array [],
            "EmptyObject": Object {},
            "Null": Null,
            "Number": Number(1),
            "String": String("..."),
        }"#
    };

    assert_eq!(debug, expected);
}

#[test]
fn test_indexing() {
    use miniserde::json::{Array, Object};
    
    // Test array indexing
    let mut arr = Array::new();
    arr.push(Value::String("first".to_string()));
    arr.push(Value::Number(json::Number::U64(42)));
    arr.push(Value::Bool(true));
    
    let array_value = Value::Array(arr);
    
    // Test valid array indexing
    match &array_value[0] {
        Value::String(s) if s == "first" => {},
        _ => panic!("Expected String('first'), got {:?}", array_value[0]),
    }
    
    match &array_value[1] {
        Value::Number(json::Number::U64(42)) => {},
        _ => panic!("Expected Number(42), got {:?}", array_value[1]),
    }
    
    match &array_value[2] {
        Value::Bool(true) => {},
        _ => panic!("Expected Bool(true), got {:?}", array_value[2]),
    }
    
    // Test out-of-bounds array indexing (should return NULL)
    match &array_value[10] {
        Value::Null => {},
        _ => panic!("Expected Null for out-of-bounds access, got {:?}", array_value[10]),
    }
    
    match &array_value[1000] {
        Value::Null => {},
        _ => panic!("Expected Null for out-of-bounds access, got {:?}", array_value[1000]),
    }
    
    // Test object indexing
    let mut obj = Object::new();
    obj.insert("name".to_string(), Value::String("Alice".to_string()));
    obj.insert("age".to_string(), Value::Number(json::Number::U64(30)));
    obj.insert("active".to_string(), Value::Bool(true));
    
    let object_value = Value::Object(obj);
    
    // Test valid object key indexing
    match &object_value["name"] {
        Value::String(s) if s == "Alice" => {},
        _ => panic!("Expected String('Alice'), got {:?}", object_value["name"]),
    }
    
    match &object_value["age"] {
        Value::Number(json::Number::U64(30)) => {},
        _ => panic!("Expected Number(30), got {:?}", object_value["age"]),
    }
    
    match &object_value["active"] {
        Value::Bool(true) => {},
        _ => panic!("Expected Bool(true), got {:?}", object_value["active"]),
    }
    
    // Test missing object key (should return NULL)
    match &object_value["missing"] {
        Value::Null => {},
        _ => panic!("Expected Null for missing key, got {:?}", object_value["missing"]),
    }
    
    match &object_value["unknown"] {
        Value::Null => {},
        _ => panic!("Expected Null for missing key, got {:?}", object_value["unknown"]),
    }
    
    // Test indexing on non-array/non-object values (should return NULL)
    match &Value::Null["key"] {
        Value::Null => {},
        _ => panic!("Expected Null when indexing Null value, got {:?}", Value::Null["key"]),
    }
    
    match &Value::Bool(true)[0] {
        Value::Null => {},
        _ => panic!("Expected Null when indexing Bool value, got {:?}", Value::Bool(true)[0]),
    }
    
    match &Value::String("test".to_string())["key"] {
        Value::Null => {},
        _ => panic!("Expected Null when indexing String value, got {:?}", Value::String("test".to_string())["key"]),
    }
    
    match &Value::Number(json::Number::U64(42))[0] {
        Value::Null => {},
        _ => panic!("Expected Null when indexing Number value, got {:?}", Value::Number(json::Number::U64(42))[0]),
    }
    
    // Test nested indexing
    let nested_json = r#"{
        "users": [
            {"name": "Alice", "settings": {"theme": "dark"}},
            {"name": "Bob", "settings": {"theme": "light"}}
        ],
        "config": {"debug": true}
    }"#;
    
    let nested_value: Value = json::from_str(nested_json).unwrap();
    
    // Test accessing nested array elements
    match &nested_value["users"][0]["name"] {
        Value::String(s) if s == "Alice" => {},
        _ => panic!("Expected String('Alice'), got {:?}", nested_value["users"][0]["name"]),
    }
    
    match &nested_value["users"][1]["settings"]["theme"] {
        Value::String(s) if s == "light" => {},
        _ => panic!("Expected String('light'), got {:?}", nested_value["users"][1]["settings"]["theme"]),
    }
    
    // Test accessing nested object values
    match &nested_value["config"]["debug"] {
        Value::Bool(true) => {},
        _ => panic!("Expected Bool(true), got {:?}", nested_value["config"]["debug"]),
    }
    
    // Test mixed valid and invalid nested indexing
    match &nested_value["users"][999]["name"] {
        Value::Null => {},
        _ => panic!("Expected Null for out-of-bounds access, got {:?}", nested_value["users"][999]["name"]),
    }
    
    match &nested_value["users"][0]["missing"] {
        Value::Null => {},
        _ => panic!("Expected Null for missing key, got {:?}", nested_value["users"][0]["missing"]),
    }
    
    match &nested_value["missing"][0]["name"] {
        Value::Null => {},
        _ => panic!("Expected Null for missing object, got {:?}", nested_value["missing"][0]["name"]),
    }
}
