use crate::profile::{Profile, ProfileSet};

pub mod skim;

pub trait SelectProfile {
    fn select_profile<'a>(&self, profiles: &'a ProfileSet) -> anyhow::Result<Option<&'a Profile>>;
}

pub struct StaticProfileSelector {
    profile_name: String,
}

impl From<String>  for StaticProfileSelector {
    fn from(profile_name: String) -> Self {
        StaticProfileSelector {profile_name}
    }
}

impl SelectProfile for StaticProfileSelector {
    fn select_profile<'a>(&self, profiles: &'a ProfileSet) -> anyhow::Result<Option<&'a Profile>> {
        if let Some(profile) = profiles.get_profile(&self.profile_name) {
            Ok(Some(profile))
        } else {
            Err(anyhow::anyhow!(
                "No profile found. profile_name:{}",
                self.profile_name
            ))
        }
    }
}
