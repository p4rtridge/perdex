use std::{fmt, marker::PhantomData};

use serde::{
    Deserialize, Deserializer, Serialize,
    de::{self, IgnoredAny, IntoDeserializer},
};
use serde_with::{DeserializeAs, SerializeAs, de::DeserializeAsWrap};

use crate::jsonld::serde::EXPECTING_NODE;

macro_rules! forward_to_into_deserializer {
    ($($name:ident($T:ty);)*) => {$(
        fn $name<E>(self, v: $T) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            T::deserialize(serde::de::IntoDeserializer::into_deserializer(v))
                // No (deserializable) element in the (single-value) set.
                // Interpret it as equivalent to `null` according to the JSON-LD data model.
                .or_else(|_: E| T::deserialize(serde::de::IntoDeserializer::into_deserializer(())))
        }
    )*};
}

/// Deserialize a single element from a JSON-LD set
///
/// It tries to deserialize each of the elements in the set
/// and return the first one that succeeds;
pub struct FirstOk<U = serde_with::Same>(PhantomData<U>);

impl<T, U> SerializeAs<T> for FirstOk<U>
where
    T: Serialize,
{
    fn serialize_as<S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        value.serialize(serializer)
    }
}

impl<'de, T, U> DeserializeAs<'de, T> for FirstOk<U>
where
    T: Deserialize<'de>,
    U: DeserializeAs<'de, T>,
{
    fn deserialize_as<D>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(Visitor(PhantomData::<T>, PhantomData::<U>))
    }
}

struct Visitor<T, U>(PhantomData<T>, PhantomData<U>);

impl<'de, T, U> de::Visitor<'de> for Visitor<T, U>
where
    T: Deserialize<'de>,
    U: DeserializeAs<'de, T>,
{
    type Value = T;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(EXPECTING_NODE)
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        loop {
            if let Ok(opt) = seq.next_element::<DeserializeAsWrap<T, U>>() {
                match opt {
                    Some(value) => {
                        while let Some(IgnoredAny) = seq.next_element()? {}
                        return Ok(value.into_inner());
                    }
                    None => return T::deserialize(().into_deserializer()),
                }
            }
        }
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let deserializer = de::value::BorrowedStrDeserializer::new(v);
        U::deserialize_as(deserializer).or_else(|_: E| T::deserialize(().into_deserializer()))
    }

    fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let deserializer = de::value::BorrowedBytesDeserializer::new(v);
        U::deserialize_as(deserializer).or_else(|_: E| T::deserialize(().into_deserializer()))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        T::deserialize(().into_deserializer())
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        U::deserialize_as(deserializer)
            .or_else(|_: D::Error| T::deserialize(().into_deserializer()))
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        T::deserialize(().into_deserializer())
    }

    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        U::deserialize_as(deserializer)
            .or_else(|_: D::Error| T::deserialize(().into_deserializer()))
    }

    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        let deserializer = de::value::MapAccessDeserializer::new(map);
        U::deserialize_as(deserializer)
            .or_else(|_: A::Error| T::deserialize(().into_deserializer()))
    }

    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
    where
        A: de::EnumAccess<'de>,
    {
        let deserializer = de::value::EnumAccessDeserializer::new(data);
        U::deserialize_as(deserializer)
            .or_else(|_: A::Error| T::deserialize(().into_deserializer()))
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
