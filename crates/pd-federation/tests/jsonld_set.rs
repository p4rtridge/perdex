use pd_federation::ap_types::jsonld;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_with::serde_as;

#[serde_as]
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(bound = "T: Serialize, T: DeserializeOwned")]
struct SetStruct<T> {
    #[serde_as(as = "jsonld::serde::Set")]
    values: Vec<T>,
}

#[test]
fn empty_sequence() {
    let de_json = r#"{"values":[]}"#;
    let de: SetStruct<String> = sonic_rs::from_str(de_json).unwrap();
    assert!(de.values.is_empty());

    let de_null_json = r#"{"values":null}"#;
    let de_null: SetStruct<String> = sonic_rs::from_str(de_null_json).unwrap();
    assert!(de_null.values.is_empty());

    let ser = sonic_rs::to_string(&de).unwrap();
    assert_eq!(ser, r#"{"values":null}"#);
}

#[test]
fn single_element() {
    let json = r#"{"values":"value"}"#;
    let de: SetStruct<String> = sonic_rs::from_str(json).unwrap();
    assert_eq!(de.values, vec!["value"]);

    let ser = sonic_rs::to_string(&de).unwrap();
    assert_eq!(ser, r#"{"values":["value"]}"#);
}

#[test]
fn sequence_of_elements() {
    let json = r#"{"values":["val1","val2"]}"#;
    let de: SetStruct<String> = sonic_rs::from_str(json).unwrap();
    assert_eq!(de.values, vec!["val1", "val2"]);

    let ser = sonic_rs::to_string(&de).unwrap();
    assert_eq!(ser, json);
}
