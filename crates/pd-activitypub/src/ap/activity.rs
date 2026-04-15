use serde::{Deserialize, Serialize};
use serde_with::{serde_as, skip_serializing_none};
use sonic_rs::Value;
use time::OffsetDateTime;

use crate::{ap::object::Object, jsonld};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum ActivityType {
    Create,
    Update,
    Delete,
    Follow,
    Accept,
    Reject,
    Like,
    Undo,
    Announce,
}

#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Activity {
    #[serde(default, rename = "@context")]
    pub context: Value,

    pub id: String,

    #[serde_as(as = "jsonld::serde::FirstOk")]
    pub r#type: ActivityType,

    #[serde_as(as = "jsonld::serde::FirstOk<jsonld::serde::Id>")]
    pub actor: String,

    pub object: ObjectField,

    #[serde(with = "time::serde::iso8601")]
    pub published: OffsetDateTime,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ObjectField {
    Activity(Box<Activity>),
    Object(Box<Object>),
}
