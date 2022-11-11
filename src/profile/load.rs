use async_trait::async_trait;

use crate::profile::ProfileSet;

pub mod aws_sdk;

#[async_trait]
pub trait LoadProfiles {
    async fn load_profiles(&self) -> anyhow::Result<ProfileSet>;
}
