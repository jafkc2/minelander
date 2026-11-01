use serde::{Deserialize, Serialize};
use uuid::Uuid;

// basic account info for display and saving to config
#[derive(Serialize, Deserialize)]
pub struct AccountInfo {
    pub username: String,
    pub offline: bool,
}

// full account info for use in launching minecraft
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Account {
    pub username: String,
    pub offline: bool,
    pub uuid: String,
    pub access_token: String,
}

impl Account {
    pub fn new_offline(name: String) -> Self {
        let uuid = generate_uuid(&name);
        Account {
            username: name,
            offline: true,
            uuid,
            access_token: String::new(),
        }
    }
}

fn generate_uuid(username: &str) -> String {
    let hash = md5::compute(username.as_bytes());
    let uuid = Uuid::from_slice(hash.as_slice()).unwrap();
    uuid.to_string()
}
