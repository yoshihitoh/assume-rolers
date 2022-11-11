use chrono::{DateTime, Utc};

pub struct Credentials {
    pub key: String,
    pub secret: String,
    pub token: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl Credentials {
    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn secret(&self) -> &str {
        &self.secret
    }

    pub fn token(&self) -> Option<&str> {
        self.token.as_ref().map(|s| s.as_str())
    }

    pub fn expires_at(&self) -> Option<DateTime<Utc>> {
        self.expires_at
    }
}
