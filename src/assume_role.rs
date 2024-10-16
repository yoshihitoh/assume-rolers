use async_trait::async_trait;

use crate::mfa::ReadMfaToken;
use crate::profile::Profile;
use assume_rolers_schema::credentials::Credentials;

pub mod aws_sdk;

pub mod defaults {
    pub const DURATION_SECONDS: i32 = 3600;
}

pub struct AssumeRoleResult {
    pub credentials: Credentials,
    pub region_name: String,
}

#[async_trait]
pub trait AssumeRole {
    async fn assume_role<R: ReadMfaToken + Send + Sync + 'static>(
        &self,
        profile: &Profile,
        mfa_reader: R,
    ) -> anyhow::Result<AssumeRoleResult>;
}
