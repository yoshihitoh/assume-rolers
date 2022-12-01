use async_trait::async_trait;
use chrono::{Duration, Utc};
use reqwest::Url;
use serde::{Deserialize, Serialize};

use crate::handler::HandleCredentials;

use assume_rolers_schema::credentials::{Credentials, ProfileCredentials};

mod endpoints {
    pub const FEDERATION: &str = "https://signin.aws.amazon.com/federation";
}

pub struct FederationHandler;

#[async_trait]
impl HandleCredentials for FederationHandler {
    async fn handle_credentials(self, credentials: ProfileCredentials) -> anyhow::Result<()> {
        let expires_at = credentials
            .credentials
            .expires_at
            .ok_or_else(|| anyhow::anyhow!("expires_at is missing."))?;

        let session = FederatedSession::try_from(credentials.credentials)?;
        let session_duration = expires_at - Utc::now();

        let client = FederationClient;
        let signin_token = client.signin_token(session, session_duration).await?;
        let url = client.signin_url(signin_token)?;

        println!("{}", url);
        Ok(())
    }
}

#[derive(Serialize)]
struct FederatedSession {
    #[serde(rename = "sessionId")]
    id: String,

    #[serde(rename = "sessionKey")]
    key: String,

    #[serde(rename = "sessionToken")]
    token: String,
}

#[derive(Debug, Deserialize)]
struct SigninToken(String);

#[derive(Debug, Deserialize)]
struct FederatedResponse {
    #[serde(rename = "SigninToken")]
    signin_token: SigninToken,
}

impl TryFrom<Credentials> for FederatedSession {
    type Error = anyhow::Error;

    fn try_from(credentials: Credentials) -> Result<Self, Self::Error> {
        if let Some(token) = credentials.token {
            Ok(FederatedSession {
                id: credentials.key,
                key: credentials.secret,
                token,
            })
        } else {
            Err(anyhow::anyhow!(
                "session-token or expiration-datetime is missing."
            ))
        }
    }
}

struct FederationClient;

impl FederationClient {
    pub async fn signin_token(
        &self,
        session: FederatedSession,
        session_duration: Duration,
    ) -> anyhow::Result<SigninToken> {
        let session = serde_json::to_string(&session)?;
        let session_duration = session_duration.num_seconds().to_string();
        let query = [
            ("Action", "getSigninToken".to_string()),
            ("SessionDuration", session_duration),
            ("Session", session),
        ];

        let client = reqwest::Client::new();
        let signin_endpoint = endpoints::FEDERATION.parse::<Url>()?;
        let response = client.get(signin_endpoint).query(&query).send().await?;
        let response = serde_json::from_str::<FederatedResponse>(&response.text().await?)?;
        Ok(response.signin_token)
    }

    pub fn signin_url(&self, signin_token: SigninToken) -> anyhow::Result<Url> {
        let query = [
            ("Action", "login".to_string()),
            ("Issuer", "".to_string()),
            ("Destination", "https://console.aws.amazon.com/".to_string()),
            ("SigninToken", signin_token.0),
        ];

        let url = Url::parse_with_params(endpoints::FEDERATION, query)?;
        Ok(url)
    }
}
