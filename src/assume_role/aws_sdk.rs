use crate::assume_role::{defaults, AssumeRole, AssumeRoleResult};
use crate::mfa::ReadMfaToken;
use crate::profile::Profile;
use anyhow::bail;
use assume_rolers_schema::credentials::Credentials;
use async_trait::async_trait;
use aws_config::BehaviorVersion;
use aws_sdk_sts::config::ProvideCredentials;
use aws_sdk_sts::types::{PolicyDescriptorType, Tag};
use aws_types::region::Region;
use chrono::{Duration, Utc};

pub struct AwsSdkAssumeRole;

struct AssumeRoleInput {
    role_arn: String,
    role_session_name: String,
    policy_arns: Vec<String>,
    policy: Option<String>,
    duration_seconds: i32,
    tags: Vec<(String, String)>,
    external_id: Option<String>,
    mfa_serial: Option<String>,
    token_code: Option<String>,
}

impl AssumeRoleInput {
    async fn send(self, client: aws_sdk_sts::Client) -> anyhow::Result<AssumeRoleResult> {
        let mut builder = client
            .assume_role()
            .role_arn(self.role_arn)
            .role_session_name(self.role_session_name)
            .duration_seconds(self.duration_seconds);

        builder = self.policy_arns.into_iter().fold(builder, |b, arn| {
            b.policy_arns(PolicyDescriptorType::builder().arn(arn).build())
        });

        builder = self
            .policy
            .into_iter()
            .fold(builder, |builder, policy| builder.policy(policy));

        let tags = self
            .tags
            .into_iter()
            .map(|(k, v)| Tag::builder().key(k).value(v).build())
            .collect::<Result<Vec<_>, _>>()?;
        builder = tags
            .into_iter()
            .fold(builder, |builder, tag| builder.tags(tag));

        builder = self
            .external_id
            .into_iter()
            .fold(builder, |builder, external_id| {
                builder.external_id(external_id)
            });

        builder = self
            .mfa_serial
            .into_iter()
            .fold(builder, |builder, mfa_serial| {
                builder.serial_number(mfa_serial)
            });

        builder = self
            .token_code
            .into_iter()
            .fold(builder, |builder, token_code| {
                builder.token_code(token_code)
            });

        let expires_at = Utc::now() + Duration::seconds(i64::from(self.duration_seconds));
        let output = builder.send().await?;
        let creds = output
            .credentials
            .ok_or_else(|| anyhow::anyhow!("assume-role didn't return a credential"))?;

        let region_name = client
            .config()
            .region()
            .map(|r| r.to_string())
            .unwrap_or_default();

        Ok(AssumeRoleResult {
            credentials: Credentials {
                key: creds.access_key_id,
                secret: creds.secret_access_key,
                token: Some(creds.session_token),
                expires_at: Some(expires_at),
            },
            region_name,
        })
    }
}

impl AwsSdkAssumeRole {
    async fn sts_assume_role(
        &self,
        profile: &Profile,
        input: AssumeRoleInput,
    ) -> anyhow::Result<AssumeRoleResult> {
        let region = Region::new(profile.region_name().unwrap_or("us-east1").to_string());

        let mut loader = aws_config::defaults(BehaviorVersion::v2024_03_28()).region(region);
        if let Some(source_profile_name) = profile.source_profile_name() {
            loader = loader.profile_name(source_profile_name);
        }

        let config = loader.load().await;
        let client = aws_sdk_sts::Client::new(&config);
        let result = input.send(client).await?;
        Ok(result)
    }

    async fn credentials_provider(&self, profile: &Profile) -> anyhow::Result<AssumeRoleResult> {
        let config = aws_config::defaults(BehaviorVersion::v2024_03_28())
            .profile_name(profile.name())
            .load()
            .await;

        let credentials_provider = config
            .credentials_provider()
            .ok_or_else(|| anyhow::anyhow!("no credentials provider found"))?;

        let duration_seconds = profile
            .duration_seconds()
            .map(i32::try_from)
            .unwrap_or_else(|| Ok(defaults::DURATION_SECONDS))?;

        let expires_at = Utc::now() + Duration::seconds(i64::from(duration_seconds));

        let region_name = profile
            .region_name()
            .map(|s| s.to_string())
            .unwrap_or_default();

        let creds = credentials_provider.provide_credentials().await?;
        Ok(AssumeRoleResult {
            credentials: Credentials {
                key: creds.access_key_id().to_string(),
                secret: creds.secret_access_key().to_string(),
                token: creds.session_token().map(|s| s.to_string()),
                expires_at: Some(expires_at),
            },
            region_name,
        })
    }
}

#[async_trait]
impl AssumeRole for AwsSdkAssumeRole {
    async fn assume_role<R: ReadMfaToken + Send + Sync + 'static>(
        &self,
        profile: &Profile,
        mfa_reader: R,
    ) -> anyhow::Result<AssumeRoleResult> {
        // Since AWS SDK for Rust does not support MFA token code,
        // we need to assume-role manually if the profile has `mfa_serial`.
        // Otherwise, we can use SharedCredentialsProvider.

        if profile.role_arn().is_none() {
            bail!(
                "The profile \"{}\" does not have a role ARN",
                profile.name()
            );
        }

        let result = if let Some(mfa_serial) = profile.mfa_serial() {
            let input = AssumeRoleInput {
                role_arn: profile.role_arn().unwrap().to_string(),
                role_session_name: profile
                    .role_session_name()
                    .unwrap_or("assume-rolers-cli")
                    .to_string(),
                policy_arns: Vec::default(), // TODO
                policy: None,                // TODO
                duration_seconds: profile
                    .duration_seconds()
                    .map(i32::try_from)
                    .unwrap_or(Ok(defaults::DURATION_SECONDS))?,
                tags: Vec::default(), // TODO
                external_id: profile.external_id().map(|s| s.to_string()),
                mfa_serial: profile.mfa_serial().map(|s| s.to_string()),
                token_code: Some(mfa_reader.read_mfa_token(mfa_serial).await?),
            };
            self.sts_assume_role(profile, input).await?
        } else {
            self.credentials_provider(profile).await?
        };

        Ok(result)
    }
}
