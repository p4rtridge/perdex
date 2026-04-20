use pd_federation::ap_types::jsonld;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_with::serde_as;

#[serde_as]
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(bound = "T: Serialize, T: DeserializeOwned")]
struct SetSkipNoneStruct<T> {
    #[serde_as(as = "jsonld::serde::Set<_, jsonld::serde::SkipNone>")]
    values: Vec<Option<T>>,
}

#[test]
fn empty_sequence() {
    let de_json = r#"{"values":[]}"#;
    let de: SetSkipNoneStruct<String> = sonic_rs::from_str(de_json).unwrap();
    assert!(de.values.is_empty());

    let ser = sonic_rs::to_string(&de).unwrap();
    assert_eq!(ser, r#"{"values":null}"#);
}

#[test]
fn skips_null_elements() {
    // Deserialization maps null to None, which the test checks
    let de_json = r#"{"values":[null,"val1",null,"val2"]}"#;
    let de: SetSkipNoneStruct<String> = sonic_rs::from_str(de_json).unwrap();
    assert_eq!(
        de.values,
        vec![Some("val1".to_string()), Some("val2".to_string())]
    );

    // Serialization filters out Nones but emits an array
    let ser_data = SetSkipNoneStruct::<String> {
        values: vec![None, Some("val1".to_string()), None],
    };
    let ser = sonic_rs::to_string(&ser_data).unwrap();
    assert_eq!(ser, r#"{"values":["val1"]}"#);
}
