use async_trait::async_trait;

use crate::credentials::Credentials;
use crate::mfa::ReadMfaToken;
use crate::profile::Profile;

pub mod rusoto;

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
