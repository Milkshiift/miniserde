#![allow(clippy::derive_partial_eq_without_eq)]

use miniserde::{json, Deserialize, Serialize};

#[derive(PartialEq, Debug, Serialize, Deserialize)]
enum Tag {
    A,
    #[serde(rename = "renamedB")]
    B,
    #[allow(non_camel_case_types)]
    r#enum,
}

#[derive(PartialEq, Debug, Serialize, Deserialize)]
struct Example {
    x: String,
    t1: Tag,
    t2: Box<Tag>,
    t3: [Tag; 1],
    r#struct: Box<Nested>,
}

#[derive(PartialEq, Debug, Serialize, Deserialize)]
struct Nested {
    #[serde(skip_serializing_if = "Option::is_none")]
    y: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    z: Option<String>,
}

#[test]
fn test_de() {
    let j =
        r#" {"x": "X", "t1": "A", "t2": "renamedB", "t3": ["enum"], "struct": {"y": ["Y", "Y"]}} "#;
    let actual: Example = json::from_str(j).unwrap();
    let expected = Example {
        x: "X".to_owned(),
        t1: Tag::A,
        t2: Box::new(Tag::B),
        t3: [Tag::r#enum],
        r#struct: Box::new(Nested {
            y: Some(vec!["Y".to_owned(), "Y".to_owned()]),
            z: None,
        }),
    };
    assert_eq!(actual, expected);
}

#[derive(PartialEq, Debug, Serialize, Deserialize)]
struct DefaultExample {
    required: String,
    #[serde(default)]
    with_default: u32,
    #[serde(default = "custom_default")]
    with_custom_default: String,
    #[serde(default)]
    optional: Option<Vec<String>>,
}

fn custom_default() -> String {
    "default_value".to_string()
}

#[test]
fn test_default_field_missing() {
    let j = r#"{"required": "test"}"#;
    let actual: DefaultExample = json::from_str(j).unwrap();
    let expected = DefaultExample {
        required: "test".to_string(),
        with_default: 0,
        with_custom_default: "default_value".to_string(),
        optional: None,
    };
    assert_eq!(actual, expected);
}

#[test]
fn test_default_field_present() {
    let j = r#"{"required": "test", "with_default": 42, "with_custom_default": "custom", "optional": ["a", "b"]}"#;
    let actual: DefaultExample = json::from_str(j).unwrap();
    let expected = DefaultExample {
        required: "test".to_string(),
        with_default: 42,
        with_custom_default: "custom".to_string(),
        optional: Some(vec!["a".to_string(), "b".to_string()]),
    };
    assert_eq!(actual, expected);
}

#[derive(PartialEq, Debug, Serialize, Deserialize)]
#[serde(default)]
struct ContainerDefaultExample {
    name: String,
    value: i32,
    enabled: bool,
}

impl Default for ContainerDefaultExample {
    fn default() -> Self {
        ContainerDefaultExample {
            name: "container_default".to_string(),
            value: 999,
            enabled: true,
        }
    }
}

#[test]
fn test_container_default_missing() {
    let j = r#"{"name": "partial"}"#;
    let actual: ContainerDefaultExample = json::from_str(j).unwrap();
    let expected = ContainerDefaultExample {
        name: "partial".to_string(),
        value: 999,
        enabled: true,
    };
    assert_eq!(actual, expected);
}

#[test]
fn test_container_default_complete() {
    let j = r#"{"name": "complete", "value": 123, "enabled": false}"#;
    let actual: ContainerDefaultExample = json::from_str(j).unwrap();
    let expected = ContainerDefaultExample {
        name: "complete".to_string(),
        value: 123,
        enabled: false,
    };
    assert_eq!(actual, expected);
}

#[derive(PartialEq, Debug, Serialize, Deserialize)]
#[serde(default = "create_default_config")]
struct CustomContainerDefaultExample {
    setting1: String,
    setting2: i32,
}

fn create_default_config() -> CustomContainerDefaultExample {
    CustomContainerDefaultExample {
        setting1: "custom_default".to_string(),
        setting2: 42,
    }
}

#[test]
fn test_container_custom_default_missing() {
    let j = r#"{"setting1": "custom"}"#;
    let actual: CustomContainerDefaultExample = json::from_str(j).unwrap();
    let expected = CustomContainerDefaultExample {
        setting1: "custom".to_string(),
        setting2: 42,
    };
    assert_eq!(actual, expected);
}

#[test]
fn test_ser() {
    let example = Example {
        x: "X".to_owned(),
        t1: Tag::A,
        t2: Box::new(Tag::B),
        t3: [Tag::r#enum],
        r#struct: Box::new(Nested {
            y: Some(vec!["Y".to_owned(), "Y".to_owned()]),
            z: None,
        }),
    };
    let actual = json::to_string(&example);
    let expected =
        r#"{"x":"X","t1":"A","t2":"renamedB","t3":["enum"],"struct":{"y":["Y","Y"]}}"#;
    assert_eq!(actual, expected);
}
