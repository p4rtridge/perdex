use pd_federation::ap_type::jsonld;
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
    let de_json = r#"{"values":[null,"val1",null,"val2"]}"#;
    let de: SetSkipNoneStruct<String> = sonic_rs::from_str(de_json).unwrap();
    assert_eq!(
        de.values,
        vec![Some("val1".to_string()), Some("val2".to_string())]
    );

    let ser_data = SetSkipNoneStruct::<String> {
        values: vec![None, Some("val1".to_string()), None],
    };
    let ser = sonic_rs::to_string(&ser_data).unwrap();
    assert_eq!(ser, r#"{"values":["val1"]}"#);
}

macro_rules! test_primitive {
    ($name:ident, $type:ty, $json:expr, $expected:expr) => {
        #[test]
        fn $name() {
            let json = format!(r#"{{"values":[null, {}]}}"#, $json);
            let de: SetSkipNoneStruct<$type> = sonic_rs::from_str(&json).unwrap();
            assert_eq!(de.values, vec![Some($expected)]);
            let ser_json = format!(r#"{{"values":[{}]}}"#, $json);
            let ser = sonic_rs::to_string(&de).unwrap();
            assert_eq!(ser, ser_json);
        }
    };
}

test_primitive!(test_bool, bool, "true", true);
test_primitive!(test_i8, i8, "-42", -42i8);
test_primitive!(test_i16, i16, "-42", -42i16);
test_primitive!(test_i32, i32, "-42", -42i32);
test_primitive!(test_i64, i64, "-42", -42i64);
test_primitive!(test_u8, u8, "42", 42u8);
test_primitive!(test_u16, u16, "42", 42u16);
test_primitive!(test_u32, u32, "42", 42u32);
test_primitive!(test_u64, u64, "42", 42u64);
test_primitive!(test_f32, f32, "42.5", 42.5f32);
test_primitive!(test_f64, f64, "42.5", 42.5f64);
test_primitive!(test_json_char, char, "\"x\"", 'x');
