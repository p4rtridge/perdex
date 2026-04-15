use std::marker::PhantomData;

use serde::{
    Deserialize,
    de::{self},
};
use serde_with::{DeserializeAs, SerializeAs, de::DeserializeAsWrap, ser::SerializeAsWrap};

use crate::jsonld::serde::EXPECTING_SET;

macro_rules! forward_to_into_deserializer {
    (
        @some $name:ident($ty:ty);
        $($rest:tt)*
    ) => {
        fn $name<E>(self, v: $ty) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            let deserializer = serde::de::IntoDeserializer::into_deserializer(v);
            let value = U::deserialize_as(deserializer)?;
            Ok(vec![Some(value)])
        }
        forward_to_into_deserializer! { $($rest)* }
    };
    (
        $name:ident($ty:ty);
        $($rest:tt)*
    ) => {
        fn $name<E>(self, v: $ty) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            let deserializer = serde::de::IntoDeserializer::into_deserializer(v);
            let value = U::deserialize_as(deserializer)?;
            Ok(vec![value])
        }
        forward_to_into_deserializer! { $($rest)* }
    };
    () => {};
}

pub struct SkipNone;
pub struct KeepNone;

/// Deserialize a single value or a set to a [`Vec`].
///
/// It tries to deserilize @id attribute if the value is a object
/// according to the JSON-LD data model
pub struct Set<U = serde_with::Same, F = KeepNone>(PhantomData<U>, PhantomData<F>);

impl<T, U> SerializeAs<Vec<T>> for Set<U>
where
    U: SerializeAs<T>,
{
    fn serialize_as<S>(source: &Vec<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if source.is_empty() {
            serializer.serialize_none()
        } else {
            let iter = source.iter().map(|item| SerializeAsWrap::<T, U>::new(item));
            serializer.collect_seq(iter)
        }
    }
}

impl<T, U> SerializeAs<Vec<Option<T>>> for Set<U, SkipNone>
where
    U: SerializeAs<T>,
{
    fn serialize_as<S>(source: &Vec<Option<T>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let iter = source
            .iter()
            .flatten()
            .map(|item| SerializeAsWrap::<T, U>::new(item));
        serializer.collect_seq(iter)
    }
}

impl<'de, T, U> DeserializeAs<'de, Vec<T>> for Set<U>
where
    T: Deserialize<'de>,
    U: DeserializeAs<'de, T>,
{
    fn deserialize_as<D>(deserializer: D) -> Result<Vec<T>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(Visitor(PhantomData::<T>, PhantomData::<U>))
    }
}

impl<'de, T, U> DeserializeAs<'de, Vec<Option<T>>> for Set<U, SkipNone>
where
    T: Deserialize<'de>,
    U: DeserializeAs<'de, T>,
{
    fn deserialize_as<D>(deserializer: D) -> Result<Vec<Option<T>>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(SkipNoneVisitor(PhantomData::<T>, PhantomData::<U>))
    }
}

struct Visitor<T, U>(PhantomData<T>, PhantomData<U>);

impl<'de, T, U> serde::de::Visitor<'de> for Visitor<T, U>
where
    T: Deserialize<'de>,
    U: DeserializeAs<'de, T>,
{
    type Value = Vec<T>;

    fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(EXPECTING_SET)
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        let mut values = Vec::with_capacity(seq.size_hint().unwrap_or(0));
        while let Some(value) = seq.next_element::<DeserializeAsWrap<T, U>>()? {
            values.push(value.into_inner());
        }
        Ok(values)
    }

    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        let deserializer = serde::de::value::MapAccessDeserializer::new(map);
        let value = U::deserialize_as(deserializer)?;
        Ok(vec![value])
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let deserializer = serde::de::value::BorrowedStrDeserializer::new(v);
        let value = U::deserialize_as(deserializer)?;
        Ok(vec![value])
    }

    fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let deserializer = serde::de::value::BorrowedBytesDeserializer::new(v);
        let value = U::deserialize_as(deserializer)?;
        Ok(vec![value])
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Vec::new())
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = U::deserialize_as(deserializer)?;
        Ok(vec![value])
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Vec::new())
    }

    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = U::deserialize_as(deserializer)?;
        Ok(vec![value])
    }

    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
    where
        A: de::EnumAccess<'de>,
    {
        let deserializer = serde::de::value::EnumAccessDeserializer::new(data);
        let value = U::deserialize_as(deserializer)?;
        Ok(vec![value])
    }

    forward_to_into_deserializer! {
        visit_bool(bool);
        visit_i8(i8);
        visit_i16(i16);
        visit_i32(i32);
        visit_i64(i64);
        visit_i128(i128);
        visit_u8(u8);
        visit_u16(u16);
        visit_u32(u32);
        visit_u64(u64);
        visit_u128(u128);
        visit_f32(f32);
        visit_f64(f64);
        visit_char(char);
        visit_str(&str);
        visit_string(String);
        visit_bytes(&[u8]);
        visit_byte_buf(Vec<u8>);
    }
}

struct SkipNoneVisitor<T, U>(PhantomData<T>, PhantomData<U>);

impl<'de, T, U> serde::de::Visitor<'de> for SkipNoneVisitor<T, U>
where
    T: Deserialize<'de>,
    U: DeserializeAs<'de, T>,
{
    type Value = Vec<Option<T>>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(EXPECTING_SET)
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let deserializer = serde::de::value::BorrowedStrDeserializer::new(v);
        let value = U::deserialize_as(deserializer)?;
        Ok(vec![Some(value)])
    }

    fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let deserializer = serde::de::value::BorrowedBytesDeserializer::new(v);
        let value = U::deserialize_as(deserializer)?;
        Ok(vec![Some(value)])
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Vec::new())
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = U::deserialize_as(deserializer)?;
        Ok(vec![Some(value)])
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Vec::new())
    }

    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = U::deserialize_as(deserializer)?;
        Ok(vec![Some(value)])
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        let mut values = Vec::with_capacity(seq.size_hint().unwrap_or(0));
        while let Some(value) = seq.next_element::<Option<DeserializeAsWrap<T, U>>>()? {
            if let Some(value) = value {
                values.push(Some(value.into_inner()));
            }
        }
        Ok(values)
    }

    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        let deserializer = serde::de::value::MapAccessDeserializer::new(map);
        let value = U::deserialize_as(deserializer)?;
        Ok(vec![Some(value)])
    }

    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
    where
        A: de::EnumAccess<'de>,
    {
        let deserializer = serde::de::value::EnumAccessDeserializer::new(data);
        let value = U::deserialize_as(deserializer)?;
        Ok(vec![Some(value)])
    }

    forward_to_into_deserializer! {
        @some visit_bool(bool);
        @some visit_i8(i8);
        @some visit_i16(i16);
        @some visit_i32(i32);
        @some visit_i64(i64);
        @some visit_i128(i128);
        @some visit_u8(u8);
        @some visit_u16(u16);
        @some visit_u32(u32);
        @some visit_u64(u64);
        @some visit_u128(u128);
        @some visit_f32(f32);
        @some visit_f64(f64);
        @some visit_char(char);
        @some visit_str(&str);
        @some visit_string(String);
        @some visit_bytes(&[u8]);
        @some visit_byte_buf(Vec<u8>);
    }
}
