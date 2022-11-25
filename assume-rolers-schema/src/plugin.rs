use std::io::{self, Read};

use serde::{Deserialize, Serialize};

use crate::credentials::ProfileCredentials;
use crate::shell::Shell;

#[derive(Serialize, Deserialize)]
pub struct PluginPayload {
    pub version: String,
    pub shell: Option<Shell>,
    pub credentials: ProfileCredentials,
}

impl PluginPayload {
    pub fn new(shell: Option<Shell>, credentials: ProfileCredentials) -> PluginPayload {
        PluginPayload {
            version: env!("CARGO_PKG_VERSION").to_string(),
            shell,
            credentials,
        }
    }

    pub fn from_stdin() -> anyhow::Result<PluginPayload> {
        let mut json = String::new();
        io::stdin().read_to_string(&mut json)?;

        let payload = serde_json::from_str(&json)?;
        Ok(payload)
    }
}
