use std::marker::PhantomData;

use serde::{
    Deserialize, Deserializer, Serialize,
    de::{self, IgnoredAny, IntoDeserializer, value::SeqAccessDeserializer},
};
use serde_with::{DeserializeAs, SerializeAs, de::DeserializeAsWrap};

use crate::ap_types::jsonld::serde::EXPECTING_NODE;

/// Deserialize a single node identifier string or a set of node identifier strings.
pub struct Id;

impl<T> SerializeAs<T> for Id
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

impl<'de, T> DeserializeAs<'de, T> for Id
where
    T: Deserialize<'de>,
{
    fn deserialize_as<D>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(Visitor(PhantomData::<T>))
    }
}

struct Visitor<T>(PhantomData<T>);

impl<'de, T> de::Visitor<'de> for Visitor<T>
where
    T: Deserialize<'de>,
{
    type Value = T;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(EXPECTING_NODE)
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "camelCase")]
        enum Key {
            #[serde(alias = "@id")]
            Id,
            #[serde(other)]
            Other,
        }

        while let Some(key) = map.next_key()? {
            match key {
                Key::Id => {
                    let value = map.next_value()?;
                    while let Some((IgnoredAny, IgnoredAny)) = map.next_entry()? {}
                    return Ok(value);
                }
                Key::Other => {
                    let IgnoredAny = map.next_value()?;
                }
            }
        }

        Err(de::Error::missing_field("id"))
    }

    fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        struct SeqAccess<A>(A);

        impl<'de, A> de::SeqAccess<'de> for SeqAccess<A>
        where
            A: de::SeqAccess<'de>,
        {
            type Error = A::Error;

            fn next_element<T>(&mut self) -> Result<Option<T>, Self::Error>
            where
                T: Deserialize<'de>,
            {
                let value = self.0.next_element::<DeserializeAsWrap<T, Id>>()?;
                Ok(value.map(DeserializeAsWrap::into_inner))
            }

            fn next_element_seed<T>(&mut self, _seed: T) -> Result<Option<T::Value>, Self::Error>
            where
                T: de::DeserializeSeed<'de>,
            {
                unreachable!()
            }

            fn size_hint(&self) -> Option<usize> {
                self.0.size_hint()
            }
        }

        T::deserialize(SeqAccessDeserializer::new(SeqAccess(seq)))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        T::deserialize(v.into_deserializer())
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        T::deserialize(de::value::BorrowedStrDeserializer::new(v))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        T::deserialize(v.into_deserializer())
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        T::deserialize(v.into_deserializer())
    }

    fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        T::deserialize(de::value::BorrowedBytesDeserializer::new(v))
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        T::deserialize(v.into_deserializer())
    }
}
