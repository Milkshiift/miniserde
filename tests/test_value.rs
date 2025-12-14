#![allow(clippy::uninlined_format_args)]

use indoc::indoc;
use miniserde::json::{self, Value, Array, Number, Object};

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

#[test]
fn test_accessor_methods() {
    // Test as_bool method
    let bool_true = Value::Bool(true);
    let bool_false = Value::Bool(false);
    assert_eq!(bool_true.as_bool(), Some(true));
    assert_eq!(bool_false.as_bool(), Some(false));
    
    // Test as_bool on wrong types
    assert_eq!(Value::Null.as_bool(), None);
    assert_eq!(Value::String("test".to_string()).as_bool(), None);
    assert_eq!(Value::Number(Number::U64(1)).as_bool(), None);
    assert_eq!(Value::Array(Array::new()).as_bool(), None);
    assert_eq!(Value::Object(Object::new()).as_bool(), None);
    
    // Test as_str method
    let string_value = Value::String("hello world".to_string());
    assert_eq!(string_value.as_str(), Some("hello world"));
    
    // Test as_str on wrong types
    assert_eq!(Value::Null.as_str(), None);
    assert_eq!(Value::Bool(true).as_str(), None);
    assert_eq!(Value::Number(Number::U64(42)).as_str(), None);
    assert_eq!(Value::Array(Array::new()).as_str(), None);
    assert_eq!(Value::Object(Object::new()).as_str(), None);
    
    // Test as_u64 method
    let u64_value = Value::Number(Number::U64(123));
    let i64_positive = Value::Number(Number::I64(456));
    let i64_negative = Value::Number(Number::I64(-789));
    let f64_value = Value::Number(Number::F64(3.14));
    
    assert_eq!(u64_value.as_u64(), Some(123));
    assert_eq!(i64_positive.as_u64(), Some(456));
    assert_eq!(i64_negative.as_u64(), None); // negative numbers can't convert to u64
    assert_eq!(f64_value.as_u64(), None); // f64 can't convert to u64 via as_u64
    
    // Test as_u64 on wrong types
    assert_eq!(Value::Null.as_u64(), None);
    assert_eq!(Value::Bool(true).as_u64(), None);
    assert_eq!(Value::String("123".to_string()).as_u64(), None);
    assert_eq!(Value::Array(Array::new()).as_u64(), None);
    assert_eq!(Value::Object(Object::new()).as_u64(), None);
    
    // Test as_i64 method
    let i64_value = Value::Number(Number::I64(-456));
    let u64_small = Value::Number(Number::U64(123));
    let u64_large = Value::Number(Number::U64(u64::MAX));
    let f64_integer = Value::Number(Number::F64(2.0));
    
    assert_eq!(i64_value.as_i64(), Some(-456));
    assert_eq!(u64_small.as_i64(), Some(123));
    assert_eq!(u64_large.as_i64(), None); // too large for i64
    assert_eq!(f64_integer.as_i64(), None); // f64 can't convert to i64 via as_i64
    
    // Test as_i64 on wrong types
    assert_eq!(Value::Null.as_i64(), None);
    assert_eq!(Value::Bool(true).as_i64(), None);
    assert_eq!(Value::String("456".to_string()).as_i64(), None);
    assert_eq!(Value::Array(Array::new()).as_i64(), None);
    assert_eq!(Value::Object(Object::new()).as_i64(), None);
    
    // Test as_f64 method
    let u64_num = Value::Number(Number::U64(42));
    let i64_num = Value::Number(Number::I64(-17));
    let f64_num = Value::Number(Number::F64(3.14159));
    
    assert_eq!(u64_num.as_f64(), Some(42.0));
    assert_eq!(i64_num.as_f64(), Some(-17.0));
    assert_eq!(f64_num.as_f64(), Some(3.14159));
    
    // Test as_f64 on wrong types
    assert_eq!(Value::Null.as_f64(), None);
    assert_eq!(Value::Bool(true).as_f64(), None);
    assert_eq!(Value::String("3.14".to_string()).as_f64(), None);
    assert_eq!(Value::Array(Array::new()).as_f64(), None);
    assert_eq!(Value::Object(Object::new()).as_f64(), None);
    
    // Test as_array method
    let mut array = Array::new();
    array.push(Value::String("item1".to_string()));
    array.push(Value::Number(Number::U64(2)));
    let array_value = Value::Array(array);
    
    assert!(array_value.as_array().is_some());
    assert_eq!(array_value.as_array().unwrap().len(), 2);
    
    // Test as_array on wrong types
    assert!(Value::Null.as_array().is_none());
    assert!(Value::Bool(true).as_array().is_none());
    assert!(Value::String("test".to_string()).as_array().is_none());
    assert!(Value::Number(Number::U64(42)).as_array().is_none());
    assert!(Value::Object(Object::new()).as_array().is_none());
    
    // Test as_object method
    let mut object = Object::new();
    object.insert("key1".to_string(), Value::String("value1".to_string()));
    object.insert("key2".to_string(), Value::Number(Number::U64(42)));
    let object_value = Value::Object(object);
    
    assert!(object_value.as_object().is_some());
    assert_eq!(object_value.as_object().unwrap().len(), 2);
    
    // Test as_object on wrong types
    assert!(Value::Null.as_object().is_none());
    assert!(Value::Bool(true).as_object().is_none());
    assert!(Value::String("test".to_string()).as_object().is_none());
    assert!(Value::Number(Number::U64(42)).as_object().is_none());
    assert!(Value::Array(Array::new()).as_object().is_none());
}

#[test]
fn test_accessor_methods_edge_cases() {
    // Test edge cases for numeric conversions
    
    // Test u64 -> i64 conversion at boundary
    let max_i64 = i64::MAX as u64;
    let min_i64 = i64::MIN as u64;
    
    let max_i64_value = Value::Number(Number::U64(max_i64));
    let min_i64_value = Value::Number(Number::U64(min_i64));
    
    assert_eq!(max_i64_value.as_i64(), Some(i64::MAX));
    assert_eq!(min_i64_value.as_i64(), None); // overflow
    
    // Test i64 -> u64 conversion at boundary
    let max_i64_neg = Value::Number(Number::I64(-1));
    assert_eq!(max_i64_neg.as_u64(), None); // negative
    
    // Test precision loss in f64 conversions (if any)
    let large_u64 = u64::MAX;
    let large_u64_value = Value::Number(Number::U64(large_u64));
    let as_f64 = large_u64_value.as_f64().unwrap();
    
    // The f64 might not preserve all precision for very large numbers
    assert!(as_f64.is_finite());
    assert!(as_f64 > 0.0);
    
    // Test special floating point values
    let inf_value = Value::Number(Number::F64(f64::INFINITY));
    let neg_inf_value = Value::Number(Number::F64(f64::NEG_INFINITY));
    let nan_value = Value::Number(Number::F64(f64::NAN));
    
    assert_eq!(inf_value.as_f64(), Some(f64::INFINITY));
    assert_eq!(neg_inf_value.as_f64(), Some(f64::NEG_INFINITY));
    assert!(nan_value.as_f64().unwrap().is_nan());
    
    // Test empty containers
    let empty_array = Value::Array(Array::new());
    let empty_object = Value::Object(Object::new());
    
    assert!(empty_array.as_array().is_some());
    assert_eq!(empty_array.as_array().unwrap().len(), 0);
    assert!(empty_object.as_object().is_some());
    assert_eq!(empty_object.as_object().unwrap().len(), 0);
    
    // Test accessor methods return references correctly
    let string_val = Value::String("test".to_string());
    let str_ref = string_val.as_str().unwrap();
    assert_eq!(str_ref, "test");
    
    let mut array = Array::new();
    array.push(Value::Number(Number::U64(1)));
    let array_val = Value::Array(array);
    let array_ref = array_val.as_array().unwrap();
    assert_eq!(array_ref.len(), 1);
    
    let mut object = Object::new();
    object.insert("key".to_string(), Value::Bool(true));
    let object_val = Value::Object(object);
    let object_ref = object_val.as_object().unwrap();
    assert_eq!(object_ref.len(), 1);
}