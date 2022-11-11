use std::env;
use std::str::FromStr;

use async_trait::async_trait;
use chrono::Duration;
use rusoto_core::Region;
use rusoto_sts::{StsAssumeRoleSessionCredentialsProvider, StsClient};

use crate::assume_role::{AssumeRole, AssumeRoleResult};
use crate::credentials::Credentials;
use crate::mfa::ReadMfaToken;
use crate::profile::Profile;

pub struct RusotoAssumeRole;

async fn credentials_provider_for(
    profile: &Profile,
    region: Region,
) -> anyhow::Result<StsAssumeRoleSessionCredentialsProvider> {
    let role_arn = profile
        .role_arn()
        .map(Ok)
        .unwrap_or_else(|| {
            Err(anyhow::anyhow!(
                "no role_arn found. profile:{}",
                profile.name()
            ))
        })?
        .to_string();

    let session_name = profile
        .role_session_name()
        .unwrap_or("assume-rolers-cli")
        .to_string();

    let external_id = profile.external_id().map(|s| s.to_string());

    let session_duration = if let Some(d) = profile.duration_seconds {
        Some(Duration::seconds(i64::try_from(d)?))
    } else {
        None
    };
    let scope_down_policy = profile.scope_down_policy().map(|s| s.to_string());
    let mfa_serial = profile.mfa_serial().map(|s| s.to_string());

    let sts_client = StsClient::new(region);
    Ok(StsAssumeRoleSessionCredentialsProvider::new(
        sts_client,
        role_arn,
        session_name,
        external_id,
        session_duration,
        scope_down_policy,
        mfa_serial,
    ))
}

#[async_trait]
impl AssumeRole for RusotoAssumeRole {
    async fn assume_role<R: ReadMfaToken + Send + Sync + 'static>(
        &self,
        profile: &Profile,
        mfa_reader: R,
    ) -> anyhow::Result<AssumeRoleResult> {
        let source_profile = profile.source_profile_name().unwrap_or("default");
        env::set_var("AWS_PROFILE", source_profile);

        let region = profile
            .region_name()
            .map(Region::from_str)
            .unwrap_or(Ok(Region::UsWest1))?;

        let mut provider = credentials_provider_for(profile, region.clone()).await?;
        if let Some(mfa_serial) = profile.mfa_serial() {
            let code = mfa_reader.read_mfa_token(mfa_serial).await?;
            provider.set_mfa_code(code);
        }

        let credentials = provider.assume_role().await?;
        Ok(AssumeRoleResult {
            credentials: Credentials {
                key: credentials.aws_access_key_id().to_string(),
                secret: credentials.aws_secret_access_key().to_string(),
                token: credentials.token().clone(),
                expires_at: *credentials.expires_at(),
            },
            region_name: region.name().to_string(),
        })
    }
}
