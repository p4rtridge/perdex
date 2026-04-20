use pd_federation::ap_type::jsonld;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_with::serde_as;

#[serde_as]
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(bound = "T: Serialize, T: DeserializeOwned")]
struct IdStruct<T> {
    #[serde_as(as = "jsonld::serde::Id")]
    value: T,
}

#[test]
fn string_literal() {
    let json = r#"{"value":"https://example.com/users/alice"}"#;
    let de: IdStruct<String> = sonic_rs::from_str(json).unwrap();
    assert_eq!(de.value, "https://example.com/users/alice");

    let ser = sonic_rs::to_string(&de).unwrap();
    assert_eq!(ser, json);
}

#[test]
fn from_object_map_id() {
    let de_json = r#"{"value":{"id":"https://example.com/users/alice","type":"Person"}}"#;
    let de: IdStruct<String> = sonic_rs::from_str(de_json).unwrap();
    assert_eq!(de.value, "https://example.com/users/alice");
}

#[test]
fn from_object_map_at_id() {
    let de_json = r#"{"value":{"@id":"https://example.com/users/bob"}}"#;
    let de: IdStruct<String> = sonic_rs::from_str(de_json).unwrap();
    assert_eq!(de.value, "https://example.com/users/bob");
}

#[test]
fn sequence_of_ids() {
    let de_json = r#"{"value":[
        "https://example.com/users/alice",
        {"@id":"https://example.com/users/bob"},
        {"id":"https://example.com/users/carol","name":"Carol"}
    ]}"#;

    let de: IdStruct<Vec<String>> = sonic_rs::from_str(de_json).unwrap();
    assert_eq!(
        de.value,
        vec![
            "https://example.com/users/alice".to_string(),
            "https://example.com/users/bob".to_string(),
            "https://example.com/users/carol".to_string(),
        ]
    );
}
