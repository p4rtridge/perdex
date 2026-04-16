//! Integration tests for `pd_activitypub::jsonld::serde::FirstOk`

use std::{
    borrow::Cow,
    fmt::{Debug, Display},
};

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_with::serde_as;

use pd_activitypub::jsonld::serde::FirstOk;

#[serde_as]
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(bound = "T: Serialize, T: DeserializeOwned")]
struct TestStruct<T> {
    #[serde_as(as = "FirstOk")]
    value: T,
}

fn assert_test_struct<T, U>(value: T, expected_value: U)
where
    T: Serialize + DeserializeOwned,
    T: Clone + Debug + PartialEq,
    U: Debug + Display,
{
    let expected_ser_data = format!(r#"{{"value":{:?}}}"#, expected_value);
    let ser_data = TestStruct {
        value: value.clone(),
    };
    let ser_data = sonic_rs::to_string(&ser_data).unwrap();
    assert_eq!(ser_data, expected_ser_data);

    let json_str = format!(
        r#"{{"value": [{{"value": "some value"}},{:?}]}}"#,
        expected_value
    );
    let expected_de_data = TestStruct { value };
    let de_data: TestStruct<T> = sonic_rs::from_str(&json_str).unwrap();
    assert_eq!(de_data, expected_de_data);
}

#[test]
fn serde_scalar() {
    assert_test_struct(true, true);
    assert_test_struct(-42i8, -42i8);
    assert_test_struct(-42i16, -42i16);
    assert_test_struct(-42i32, -42i32);
    assert_test_struct(-42i64, -42i64);
    assert_test_struct(42u8, 42u8);
    assert_test_struct(42u16, 42u16);
    assert_test_struct(42u32, 42u32);
    assert_test_struct(42u64, 42u64);
    assert_test_struct(42.1f32, 42.1f32);
    assert_test_struct(42.1f64, 42.1f64);
    assert_test_struct('x', String::from("x"));
    assert_test_struct(String::from("some value"), String::from("some value"));
    assert_test_struct(Cow::Borrowed("x"), String::from("x"));
}
