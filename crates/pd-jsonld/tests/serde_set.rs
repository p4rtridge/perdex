//! Integration tests for `pd_jsonld::serde::Set`.
//!
//! `Set<U>` is a JSON-LD "set" adapter: when *deserializing* it accepts
//! either a bare scalar/map/enum (wraps it in a single-element `Vec`) or a
//! JSON array (collects every element).  When *serializing* an empty `Vec`
//! becomes `null`; a non-empty `Vec` becomes a JSON array.

use std::borrow::Cow;

use bytes::Bytes;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_test::{Token, assert_de_tokens, assert_ser_tokens, assert_tokens};
use serde_with::serde_as;

use pd_jsonld::serde::{Set, SkipNone};

#[serde_as]
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(bound = "T: Serialize, T: DeserializeOwned")]
pub struct TestStruct<T> {
    #[serde_as(as = "Set")]
    pub values: Vec<T>,
}

#[serde_as]
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(bound = "T: Serialize, T: DeserializeOwned")]
pub struct TestStructOpt<T> {
    #[serde_as(as = "Set<_, SkipNone>")]
    pub values: Vec<Option<T>>,
}

fn assert_test_struct<T>(value: T, expected_token: Token)
where
    T: Serialize + DeserializeOwned + PartialEq + std::fmt::Debug,
{
    let data = TestStruct::<T> {
        values: vec![value],
    };
    assert_tokens(
        &data,
        &[
            Token::Struct {
                name: "TestStruct",
                len: 1,
            },
            Token::Str("values"),
            Token::Seq { len: Some(1) },
            expected_token,
            Token::SeqEnd,
            Token::StructEnd,
        ],
    );
}

fn assert_test_struct_opt<T>(value: T, expected_token: Token)
where
    T: Serialize + DeserializeOwned + PartialEq + std::fmt::Debug + Clone,
{
    let ser_data = TestStructOpt::<T> {
        values: vec![None, Some(value.clone())],
    };
    assert_ser_tokens(
        &ser_data,
        &[
            Token::Struct {
                name: "TestStructOpt",
                len: 1,
            },
            Token::Str("values"),
            // len is None because the deserializer doesn't know how many Some/None pairs there are
            Token::Seq { len: None },
            expected_token,
            Token::SeqEnd,
            Token::StructEnd,
        ],
    );

    let de_data = TestStructOpt::<T> {
        values: vec![Some(value)],
    };
    assert_de_tokens(
        &de_data,
        &[
            Token::Struct {
                name: "TestStructOpt",
                len: 1,
            },
            Token::Str("values"),
            Token::Seq { len: Some(2) },
            Token::Some,
            expected_token,
            Token::None,
            Token::SeqEnd,
            Token::StructEnd,
        ],
    );
}

#[test]
fn serde_empty() {
    let data = TestStruct::<u32> { values: vec![] };

    assert_tokens(
        &data,
        &[
            Token::Struct {
                name: "TestStruct",
                len: 1,
            },
            Token::Str("values"),
            Token::None,
            Token::StructEnd,
        ],
    );
}

#[test]
fn serde_keep_none() {
    assert_test_struct(true, Token::Bool(true));
    assert_test_struct(-42i8, Token::I8(-42));
    assert_test_struct(-42i16, Token::I16(-42));
    assert_test_struct(-42i32, Token::I32(-42));
    assert_test_struct(-42i64, Token::I64(-42));
    assert_test_struct(42u8, Token::U8(42));
    assert_test_struct(42u16, Token::U16(42));
    assert_test_struct(42u32, Token::U32(42));
    assert_test_struct(42u64, Token::U64(42));
    assert_test_struct(42.0f32, Token::F32(42.0));
    assert_test_struct(42.0f64, Token::F64(42.0));
    assert_test_struct('x', Token::Char('x'));
    assert_test_struct(String::from("x"), Token::Str("x"));
    assert_test_struct(Cow::Borrowed("x"), Token::BorrowedStr("x"));
    assert_test_struct(Bytes::from_static(b"x"), Token::BorrowedBytes(b"x"));
    assert_test_struct::<Option<u32>>(None, Token::None);
    assert_test_struct((), Token::Unit);
}

#[test]
fn serde_skip_none() {
    assert_test_struct_opt(true, Token::Bool(true));
    assert_test_struct_opt(-42i8, Token::I8(-42));
    assert_test_struct_opt(-42i16, Token::I16(-42));
    assert_test_struct_opt(-42i32, Token::I32(-42));
    assert_test_struct_opt(-42i64, Token::I64(-42));
    assert_test_struct_opt(42u8, Token::U8(42));
    assert_test_struct_opt(42u16, Token::U16(42));
    assert_test_struct_opt(42u32, Token::U32(42));
    assert_test_struct_opt(42u64, Token::U64(42));
    assert_test_struct_opt(42.0f32, Token::F32(42.0));
    assert_test_struct_opt(42.0f64, Token::F64(42.0));
    assert_test_struct_opt('x', Token::Char('x'));
    assert_test_struct_opt(String::from("x"), Token::Str("x"));
    assert_test_struct_opt(Cow::Borrowed("x"), Token::BorrowedStr("x"));
    assert_test_struct_opt(Bytes::from_static(b"x"), Token::BorrowedBytes(b"x"));
    assert_test_struct_opt::<Option<u32>>(None, Token::None);
    assert_test_struct_opt((), Token::Unit);
}
