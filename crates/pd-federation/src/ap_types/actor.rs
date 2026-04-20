use serde::{Deserialize, Serialize};
use serde_with::{serde_as, skip_serializing_none};
use sonic_rs::Value;
use time::OffsetDateTime;

use crate::ap_types::jsonld;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum ActorType {
    Group,
    Person,
}

#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Actor {
    #[serde(default, rename = "@context")]
    pub context: Value,

    pub id: String,

    #[serde_as(as = "jsonld::serde::FirstOk")]
    pub r#type: String,

    #[serde_as(as = "Option<jsonld::serde::FirstOk>")]
    pub name: Option<String>,

    #[serde_as(as = "Option<jsonld::serde::FirstOk>")]
    pub preferred_username: Option<String>,

    #[serde(default)]
    #[serde_as(as = "jsonld::serde::FirstOk")]
    pub manually_approves_followers: bool,

    #[serde_as(as = "Option<jsonld::serde::FirstOk>")]
    pub endpoints: Option<Endpoints>,

    #[serde_as(as = "Option<jsonld::serde::FirstOk>")]
    pub inbox: Option<String>,

    #[serde_as(as = "Option<jsonld::serde::FirstOk<jsonld::serde::Id>>")]
    pub outbox: Option<String>,

    #[serde_as(as = "Option<jsonld::serde::FirstOk<jsonld::serde::Id>>")]
    pub followers: Option<String>,

    #[serde_as(as = "Option<jsonld::serde::FirstOk<jsonld::serde::Id>>")]
    pub following: Option<String>,

    #[serde_as(as = "jsonld::serde::FirstOk")]
    pub public_key: PublicKey,

    #[serde(default = "OffsetDateTime::now_utc")]
    #[serde(with = "time::serde::iso8601")]
    pub published: OffsetDateTime,
}

#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Endpoints {
    #[serde_as(as = "Option<jsonld::serde::FirstOk<jsonld::serde::Id>>")]
    pub shared_inbox: Option<String>,
}

#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicKey {
    pub id: String,

    #[serde_as(as = "jsonld::serde::FirstOk<jsonld::serde::Id>")]
    pub owner: String,

    #[serde_as(as = "jsonld::serde::FirstOk")]
    pub public_key_pem: String,
}
