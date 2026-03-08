use miniserde::{json, Deserialize, Serialize};

// ---- rename_all = "camelCase" on structs ----

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CamelCaseStruct {
    display_name: String,
    application_name: Option<String>,
    is_device: bool,
}

#[test]
fn test_serialize_rename_all_camel_case() {
    let value = CamelCaseStruct {
        display_name: "Firefox".to_string(),
        application_name: Some("firefox".to_string()),
        is_device: false,
    };

    let serialized = json::to_string(&value);
    assert!(serialized.contains("\"displayName\""));
    assert!(serialized.contains("\"applicationName\""));
    assert!(serialized.contains("\"isDevice\""));
    assert!(!serialized.contains("\"display_name\""));
    assert!(!serialized.contains("\"application_name\""));
    assert!(!serialized.contains("\"is_device\""));
}

#[test]
fn test_deserialize_rename_all_camel_case() {
    let input = r#"{"displayName":"Firefox","applicationName":"firefox","isDevice":false}"#;
    let value: CamelCaseStruct = json::from_str(input).unwrap();

    assert_eq!(value.display_name, "Firefox");
    assert_eq!(value.application_name, Some("firefox".to_string()));
    assert_eq!(value.is_device, false);
}

#[test]
fn test_deserialize_rename_all_unknown_fields_ignored() {
    let input = r#"{"displayName":"Test","applicationName":null,"isDevice":true,"extraField":42}"#;
    let value: CamelCaseStruct = json::from_str(input).unwrap();

    assert_eq!(value.display_name, "Test");
    assert_eq!(value.is_device, true);
}

// ---- rename_all with field-level rename override ----

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MixedRenameStruct {
    normal_field: String,
    #[serde(rename = "CUSTOM")]
    custom_field: u32,
}

#[test]
fn test_serialize_rename_all_with_field_override() {
    let value = MixedRenameStruct {
        normal_field: "hello".to_string(),
        custom_field: 42,
    };

    let serialized = json::to_string(&value);
    assert!(serialized.contains("\"normalField\""));
    assert!(serialized.contains("\"CUSTOM\""));
    assert!(!serialized.contains("\"normal_field\""));
    assert!(!serialized.contains("\"customField\""));
    assert!(!serialized.contains("\"custom_field\""));
}

#[test]
fn test_deserialize_rename_all_with_field_override() {
    let input = r#"{"normalField":"hello","CUSTOM":42}"#;
    let value: MixedRenameStruct = json::from_str(input).unwrap();

    assert_eq!(value.normal_field, "hello");
    assert_eq!(value.custom_field, 42);
}

// ---- alias on fields ----

#[derive(Debug, PartialEq, Deserialize)]
struct AliasStruct {
    #[serde(rename = "primary-name", alias = "primary_name")]
    primary: String,
    count: u32,
}

#[test]
fn test_deserialize_alias_primary() {
    let input = r#"{"primary-name":"abc","count":5}"#;
    let value: AliasStruct = json::from_str(input).unwrap();

    assert_eq!(value.primary, "abc");
    assert_eq!(value.count, 5);
}

#[test]
fn test_deserialize_alias_alternate() {
    let input = r#"{"primary_name":"def","count":10}"#;
    let value: AliasStruct = json::from_str(input).unwrap();

    assert_eq!(value.primary, "def");
    assert_eq!(value.count, 10);
}

// ---- alias with rename_all ----

#[derive(Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AliasWithRenameAll {
    #[serde(alias = "max_output_ports")]
    max_output_ports: Option<u32>,
}

#[test]
fn test_deserialize_alias_with_rename_all_camel() {
    let input = r#"{"maxOutputPorts":8}"#;
    let value: AliasWithRenameAll = json::from_str(input).unwrap();
    assert_eq!(value.max_output_ports, Some(8));
}

#[test]
fn test_deserialize_alias_with_rename_all_snake() {
    let input = r#"{"max_output_ports":4}"#;
    let value: AliasWithRenameAll = json::from_str(input).unwrap();
    assert_eq!(value.max_output_ports, Some(4));
}

// ---- internally tagged enum: unit variants ----

#[derive(Debug, PartialEq, Deserialize)]
#[serde(tag = "type")]
enum SimpleTagged {
    #[serde(rename = "start")]
    Start,
    #[serde(rename = "stop")]
    Stop,
}

#[test]
fn test_deserialize_tagged_enum_unit_start() {
    let input = r#"{"type":"start"}"#;
    let value: SimpleTagged = json::from_str(input).unwrap();
    assert_eq!(value, SimpleTagged::Start);
}

#[test]
fn test_deserialize_tagged_enum_unit_stop() {
    let input = r#"{"type":"stop"}"#;
    let value: SimpleTagged = json::from_str(input).unwrap();
    assert_eq!(value, SimpleTagged::Stop);
}

#[test]
fn test_deserialize_tagged_enum_unknown_tag() {
    let result = json::from_str::<SimpleTagged>(r#"{"type":"pause"}"#);
    assert!(result.is_err());
}

#[test]
fn test_deserialize_tagged_enum_missing_tag() {
    let result = json::from_str::<SimpleTagged>(r#"{"other":"value"}"#);
    assert!(result.is_err());
}

// ---- internally tagged enum: variants with named fields ----

#[derive(Debug, PartialEq, Deserialize)]
#[serde(tag = "method")]
enum Request {
    #[serde(rename = "greet")]
    Greet { id: u64, name: String },

    #[serde(rename = "add")]
    Add { id: u64, a: i32, b: i32 },

    #[serde(rename = "ping")]
    Ping { id: u64 },
}

#[test]
fn test_deserialize_tagged_enum_greet() {
    let input = r#"{"method":"greet","id":1,"name":"world"}"#;
    let value: Request = json::from_str(input).unwrap();

    assert_eq!(
        value,
        Request::Greet {
            id: 1,
            name: "world".to_string()
        }
    );
}

#[test]
fn test_deserialize_tagged_enum_add() {
    let input = r#"{"method":"add","id":2,"a":10,"b":20}"#;
    let value: Request = json::from_str(input).unwrap();

    assert_eq!(
        value,
        Request::Add {
            id: 2,
            a: 10,
            b: 20
        }
    );
}

#[test]
fn test_deserialize_tagged_enum_ping() {
    let input = r#"{"method":"ping","id":99}"#;
    let value: Request = json::from_str(input).unwrap();

    assert_eq!(value, Request::Ping { id: 99 });
}

#[test]
fn test_deserialize_tagged_enum_tag_not_first() {
    let input = r#"{"id":5,"name":"late","method":"greet"}"#;
    let value: Request = json::from_str(input).unwrap();

    assert_eq!(
        value,
        Request::Greet {
            id: 5,
            name: "late".to_string()
        }
    );
}

#[test]
fn test_deserialize_tagged_enum_extra_fields_ignored() {
    let input = r#"{"method":"ping","id":7,"extra":"ignored","another":true}"#;
    let value: Request = json::from_str(input).unwrap();

    assert_eq!(value, Request::Ping { id: 7 });
}

// ---- internally tagged enum: mixed unit and named variants ----

#[derive(Debug, PartialEq, Deserialize)]
#[serde(tag = "action")]
enum MixedAction {
    #[serde(rename = "noop")]
    Noop,

    #[serde(rename = "setValue")]
    SetValue { key: String, value: String },
}

#[test]
fn test_deserialize_tagged_enum_mixed_unit() {
    let input = r#"{"action":"noop"}"#;
    let value: MixedAction = json::from_str(input).unwrap();
    assert_eq!(value, MixedAction::Noop);
}

#[test]
fn test_deserialize_tagged_enum_mixed_named() {
    let input = r#"{"action":"setValue","key":"color","value":"red"}"#;
    let value: MixedAction = json::from_str(input).unwrap();

    assert_eq!(
        value,
        MixedAction::SetValue {
            key: "color".to_string(),
            value: "red".to_string()
        }
    );
}

// ---- internally tagged enum: fields with defaults ----

#[derive(Debug, PartialEq, Deserialize)]
#[serde(tag = "cmd")]
enum DefaultsTagged {
    #[serde(rename = "query")]
    Query {
        id: u64,
        #[serde(default)]
        verbose: bool,
        #[serde(default)]
        limit: Option<u32>,
    },
}

#[test]
fn test_deserialize_tagged_enum_defaults_present() {
    let input = r#"{"cmd":"query","id":1,"verbose":true,"limit":50}"#;
    let value: DefaultsTagged = json::from_str(input).unwrap();

    assert_eq!(
        value,
        DefaultsTagged::Query {
            id: 1,
            verbose: true,
            limit: Some(50)
        }
    );
}

#[test]
fn test_deserialize_tagged_enum_defaults_missing() {
    let input = r#"{"cmd":"query","id":2}"#;
    let value: DefaultsTagged = json::from_str(input).unwrap();

    assert_eq!(
        value,
        DefaultsTagged::Query {
            id: 2,
            verbose: false,
            limit: None
        }
    );
}

// ---- internally tagged enum: shared field names across variants ----

#[derive(Debug, PartialEq, Deserialize)]
#[serde(tag = "method")]
enum SharedFields {
    #[serde(rename = "alpha")]
    Alpha { id: u64, data: String },

    #[serde(rename = "beta")]
    Beta { id: u64, count: u32 },
}

#[test]
fn test_deserialize_tagged_enum_shared_field_alpha() {
    let input = r#"{"method":"alpha","id":1,"data":"hello"}"#;
    let value: SharedFields = json::from_str(input).unwrap();

    assert_eq!(
        value,
        SharedFields::Alpha {
            id: 1,
            data: "hello".to_string()
        }
    );
}

#[test]
fn test_deserialize_tagged_enum_shared_field_beta() {
    let input = r#"{"method":"beta","id":2,"count":42}"#;
    let value: SharedFields = json::from_str(input).unwrap();

    assert_eq!(value, SharedFields::Beta { id: 2, count: 42 });
}

// ---- internally tagged enum: field with rename ----

#[derive(Debug, PartialEq, Deserialize)]
#[serde(tag = "method")]
enum RenamedFields {
    #[serde(rename = "route")]
    Route {
        id: u64,
        #[serde(rename = "nodeIds")]
        node_ids: Vec<u32>,
    },
}

#[test]
fn test_deserialize_tagged_enum_renamed_field() {
    let input = r#"{"method":"route","id":1,"nodeIds":[10,20,30]}"#;
    let value: RenamedFields = json::from_str(input).unwrap();

    assert_eq!(
        value,
        RenamedFields::Route {
            id: 1,
            node_ids: vec![10, 20, 30]
        }
    );
}

// ---- roundtrip: serialize then deserialize with rename_all ----

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RoundTrip {
    first_name: String,
    last_name: String,
    is_active: bool,
}

#[test]
fn test_roundtrip_rename_all() {
    let original = RoundTrip {
        first_name: "Alice".to_string(),
        last_name: "Smith".to_string(),
        is_active: true,
    };

    let serialized = json::to_string(&original);
    let deserialized: RoundTrip = json::from_str(&serialized).unwrap();

    assert_eq!(original, deserialized);
}

// ---- rename_all variants ----

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
struct ScreamingSnake {
    my_field: String,
}

#[test]
fn test_serialize_screaming_snake_case() {
    let value = ScreamingSnake {
        my_field: "test".to_string(),
    };
    let serialized = json::to_string(&value);
    assert!(serialized.contains("\"MY_FIELD\""));
}

#[test]
fn test_deserialize_screaming_snake_case() {
    let input = r#"{"MY_FIELD":"test"}"#;
    let value: ScreamingSnake = json::from_str(input).unwrap();
    assert_eq!(value.my_field, "test");
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct KebabCase {
    some_value: i32,
}

#[test]
fn test_serialize_kebab_case() {
    let value = KebabCase { some_value: 7 };
    let serialized = json::to_string(&value);
    assert!(serialized.contains("\"some-value\""));
}

#[test]
fn test_deserialize_kebab_case() {
    let input = r#"{"some-value":7}"#;
    let value: KebabCase = json::from_str(input).unwrap();
    assert_eq!(value.some_value, 7);
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct PascalCaseStruct {
    my_field: String,
}

#[test]
fn test_serialize_pascal_case() {
    let value = PascalCaseStruct {
        my_field: "x".to_string(),
    };
    let serialized = json::to_string(&value);
    assert!(serialized.contains("\"MyField\""));
}

#[test]
fn test_deserialize_pascal_case() {
    let input = r#"{"MyField":"x"}"#;
    let value: PascalCaseStruct = json::from_str(input).unwrap();
    assert_eq!(value.my_field, "x");
}
