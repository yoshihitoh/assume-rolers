use std::collections::BTreeMap;
use std::convert::TryFrom;

use async_trait::async_trait;
use aws_config::profile::load;
use aws_config::profile::profile_file::ProfileFiles;
use aws_types::os_shim_internal::{Env, Fs};

use crate::profile::load::LoadProfiles;
use crate::profile::{Profile, ProfileSet};

fn profile_from(name: &str, value: &aws_config::profile::Profile) -> anyhow::Result<Profile> {
    fn s<S: Into<String>>(s: Option<S>) -> Option<String> {
        s.map(|x| x.into())
    }

    fn n(s: Option<&str>) -> anyhow::Result<Option<u32>> {
        if let Some(s) = s {
            let n = s.parse()?;
            Ok(Some(n))
        } else {
            Ok(None)
        }
    }

    Ok(Profile {
        name: name.to_string(),
        source_profile_name: s(value.get("source_profile")),
        region_name: s(value.get("region")),
        role_arn: s(value.get("role_arn")),
        role_session_name: s(value.get("role_session_name")),
        external_id: s(value.get("external_id")),
        duration_seconds: n(value.get("duration_seconds"))?,
        scope_down_policy: s(value.get("scope_down_policy")),
        mfa_serial: s(value.get("mfa_serial")),
    })
}

impl TryFrom<aws_config::profile::ProfileSet> for ProfileSet {
    type Error = anyhow::Error;

    fn try_from(value: aws_config::profile::ProfileSet) -> Result<Self, Self::Error> {
        let profiles = value
            .profiles()
            .map(|n| profile_from(n, value.get_profile(n).unwrap()).map(|p| (n.to_string(), p)))
            .collect::<Result<BTreeMap<_, _>, _>>()?;
        Ok(ProfileSet { profiles })
    }
}

#[derive(Debug, Default)]
pub struct AwsSdkProfileLoader {
    profile_files: ProfileFiles,
    fs: Fs,
    env: Env,
}

#[async_trait]
impl LoadProfiles for AwsSdkProfileLoader {
    async fn load_profiles(&self) -> anyhow::Result<ProfileSet> {
        let profiles = load(&self.fs, &self.env, &self.profile_files).await?;
        Ok(ProfileSet::try_from(profiles)?)
    }
}
