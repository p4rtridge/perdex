use std::marker::PhantomData;

use serde::{
    Deserialize, Deserializer, Serialize,
    de::{IgnoredAny, SeqAccess, Visitor, value::SeqAccessDeserializer},
};
use serde_with::{DeserializeAs, SerializeAs};

use crate::serde::EXPECTING_NODE;

pub struct Id;

impl<'de, T> DeserializeAs<'de, T> for Id
where
    T: Deserialize<'de>,
{
    fn deserialize_as<D>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(IdVisitor(PhantomData::<T>))
    }
}

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

struct IdVisitor<T>(PhantomData<T>);

impl<'de, T> Visitor<'de> for IdVisitor<T>
where
    T: Deserialize<'de>,
{
    type Value = T;

    fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(EXPECTING_NODE)
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
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

        Err(serde::de::Error::missing_field("@id"))
    }

    fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        T::deserialize(SeqAccessDeserializer::new(seq))
    }
}
