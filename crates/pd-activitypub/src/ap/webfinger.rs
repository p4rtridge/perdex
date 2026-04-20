use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Resource {
    pub subject: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub links: Vec<Link>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Link {
    pub rel: String,
    pub r#type: Option<String>,
    pub href: Option<String>,
}
