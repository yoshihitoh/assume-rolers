use std::collections::BTreeMap;

pub mod load;
pub mod select;

#[derive(Debug, Clone)]
pub struct Profile {
    pub name: String,
    pub source_profile_name: Option<String>,
    pub region_name: Option<String>,
    pub role_arn: Option<String>,
    pub role_session_name: Option<String>,
    pub external_id: Option<String>,
    pub duration_seconds: Option<u32>,
    pub scope_down_policy: Option<String>,
    pub mfa_serial: Option<String>,
}

impl Profile {
    pub fn has_role_arn(&self) -> bool {
        self.role_arn.is_some()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn source_profile_name(&self) -> Option<&str> {
        self.source_profile_name.as_deref()
    }

    pub fn region_name(&self) -> Option<&str> {
        self.region_name.as_deref()
    }

    pub fn role_arn(&self) -> Option<&str> {
        self.role_arn.as_deref()
    }

    pub fn role_session_name(&self) -> Option<&str> {
        self.role_session_name.as_deref()
    }

    pub fn external_id(&self) -> Option<&str> {
        self.external_id.as_deref()
    }

    pub fn duration_seconds(&self) -> Option<u32> {
        self.duration_seconds
    }

    pub fn scope_down_policy(&self) -> Option<&str> {
        self.scope_down_policy.as_deref()
    }

    pub fn mfa_serial(&self) -> Option<&str> {
        self.mfa_serial.as_deref()
    }
}

#[derive(Debug, Clone)]
pub struct ProfileSet {
    pub profiles: BTreeMap<String, Profile>,
}

impl ProfileSet {
    pub fn get_profile(&self, profile_name: &str) -> Option<&Profile> {
        self.profiles.get(profile_name)
    }

    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.profiles.keys().map(|k| k.as_str())
    }

    pub fn profiles(&self) -> impl Iterator<Item = &Profile> {
        self.profiles.values()
    }
}
