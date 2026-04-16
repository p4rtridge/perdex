//! Integration tests for `pd_activitypub::jsonld::serde::Id`.

use std::borrow::Cow;

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_test::{Token, assert_de_tokens, assert_tokens};
use serde_with::serde_as;

use pd_activitypub::jsonld::serde::Id;

#[serde_as]
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(bound = "T: Serialize, T: DeserializeOwned")]
struct TestStruct<T> {
    #[serde_as(as = "Id")]
    value: T,
}

fn assert_test_struct<T>(value: T, expected_token: Token)
where
    T: Serialize + DeserializeOwned + PartialEq + std::fmt::Debug,
{
    let data = TestStruct { value };

    assert_tokens(
        &data,
        &[
            Token::Struct {
                name: "TestStruct",
                len: 1,
            },
            Token::Str("value"),
            expected_token,
            Token::StructEnd,
        ],
    );
}

fn assert_deser_from_map(id_key: &'static str, first_key: &'static str, first_value: &'static str) {
    let data = TestStruct {
        value: String::from("https://example.com/users/alice"),
    };

    assert_de_tokens(
        &data,
        &[
            Token::Struct {
                name: "TestStruct",
                len: 1,
            },
            Token::Str("value"),
            Token::Map { len: Some(2) },
            Token::Str(first_key),
            Token::Str(first_value),
            Token::Str(id_key),
            Token::Str("https://example.com/users/alice"),
            Token::MapEnd,
            Token::StructEnd,
        ],
    );
}

#[test]
fn serde_scalar() {
    assert_test_struct(
        String::from("https://example.com/users/alice"),
        Token::Str("https://example.com/users/alice"),
    );
    assert_test_struct(
        Cow::Borrowed("https://example.com/users/alice"),
        Token::BorrowedStr("https://example.com/users/alice"),
    );
}

#[test]
fn serde_map_with_id_variants() {
    let cases = [("id", "type", "Person"), ("@id", "name", "Alice")];

    for (id_key, first_key, first_value) in cases {
        assert_deser_from_map(id_key, first_key, first_value);
    }
}

#[test]
fn serde_set_of_ids() {
    let data = TestStruct {
        value: vec![
            String::from("https://example.com/users/alice"),
            String::from("https://example.com/users/bob"),
            String::from("https://example.com/users/carol"),
        ],
    };

    assert_de_tokens(
        &data,
        &[
            Token::Struct {
                name: "TestStruct",
                len: 1,
            },
            Token::Str("value"),
            Token::Seq { len: Some(3) },
            Token::Str("https://example.com/users/alice"),
            Token::Map { len: Some(1) },
            Token::Str("@id"),
            Token::Str("https://example.com/users/bob"),
            Token::MapEnd,
            Token::Map { len: Some(2) },
            Token::Str("name"),
            Token::Str("Carol"),
            Token::Str("id"),
            Token::Str("https://example.com/users/carol"),
            Token::MapEnd,
            Token::SeqEnd,
            Token::StructEnd,
        ],
    );
}
