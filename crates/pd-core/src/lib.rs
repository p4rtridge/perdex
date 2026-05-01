use serde::{Deserialize, Serialize};

pub mod account;

#[derive(Debug, Deserialize, Serialize)]
pub struct PublicKey {
    pub id: String,
    pub owner: String,
    pub public_key_pem: String,
}
