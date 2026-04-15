use serde::{Deserialize, Serialize};
use serde_with::{serde_as, skip_serializing_none};
use sonic_rs::Value;
use time::OffsetDateTime;

use crate::jsonld;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum ObjectType {
    Comic,
    Chapter,
    Note,
    Image,
}

#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Object {
    #[serde(default, rename = "@context")]
    pub context: Value,

    pub id: String,

    #[serde_as(as = "jsonld::serde::FirstOk")]
    pub r#type: ObjectType,

    #[serde_as(as = "jsonld::serde::Set<jsonld::serde::Id>")]
    pub attributed_to: Vec<String>,

    #[serde_as(as = "Option<jsonld::serde::FirstOk<jsonld::serde::Id>>")]
    pub in_reply_to: Option<String>,

    #[serde_as(as = "Option<jsonld::serde::Set<jsonld::serde::Id>>")]
    pub to: Option<Vec<String>>,

    #[serde_as(as = "Option<jsonld::serde::Set<jsonld::serde::Id>>")]
    pub cc: Option<Vec<String>>,

    #[serde_as(as = "Option<jsonld::serde::FirstOk>")]
    pub name: Option<String>,

    #[serde_as(as = "Option<jsonld::serde::FirstOk>")]
    pub summary: Option<String>,

    #[serde_as(as = "Option<jsonld::serde::FirstOk>")]
    pub content: Option<String>,

    #[serde_as(as = "Option<jsonld::serde::Set>")]
    pub tag: Option<Vec<Tag>>,

    #[serde_as(as = "Option<jsonld::serde::Set>")]
    pub attachment: Option<Vec<Attachment>>,

    #[serde(with = "time::serde::iso8601")]
    pub published: OffsetDateTime,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum TagType {
    Hashtag,
    Mention,
}

#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    pub id: Option<String>,

    #[serde_as(as = "jsonld::serde::FirstOk")]
    pub r#type: TagType,

    pub href: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum AttachmentType {
    Image,
    Link,

    #[serde(other)]
    Other,
}

#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Attachment {
    #[serde_as(as = "jsonld::serde::FirstOk")]
    pub r#type: AttachmentType,

    pub media_type: Option<String>,

    pub blurhash: Option<String>,

    #[serde(alias = "href")]
    #[serde_as(as = "jsonld::serde::FirstOk")]
    pub url: String,
}
