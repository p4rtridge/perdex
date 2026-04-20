use pd_federation::ap_types::jsonld;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_with::serde_as;

#[serde_as]
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(bound = "T: Serialize, T: DeserializeOwned")]
struct FirstOkStruct<T> {
    #[serde_as(as = "jsonld::serde::FirstOk")]
    value: T,
}

#[test]
fn single_valid_value() {
    let json = r#"{"value":"expected_value"}"#;
    let de: FirstOkStruct<String> = sonic_rs::from_str(json).unwrap();
    assert_eq!(de.value, "expected_value");

    let ser = sonic_rs::to_string(&de).unwrap();
    assert_eq!(ser, json);
}

#[test]
fn fallback_sequence() {
    // Tries elements one by one; fails on complex object, succeeds on string
    let de_json = r#"{"value":[{"nested":"fail"},"expected_value"]}"#;
    let de: FirstOkStruct<String> = sonic_rs::from_str(de_json).unwrap();
    assert_eq!(de.value, "expected_value");

    let ser_json = r#"{"value":"expected_value"}"#;
    let ser = sonic_rs::to_string(&de).unwrap();
    assert_eq!(ser, ser_json);
}
