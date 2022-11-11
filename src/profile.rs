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

fn maybe_str_ref<R: AsRef<str>>(x: Option<&R>) -> Option<&str> {
    x.map(|s| s.as_ref())
}

impl Profile {
    pub fn has_role_arn(&self) -> bool {
        self.role_arn.is_some()
    }

    pub fn name(&self) -> &str {
        &self.name.as_str()
    }

    pub fn source_profile_name(&self) -> Option<&str> {
        maybe_str_ref(self.source_profile_name.as_ref())
    }

    pub fn region_name(&self) -> Option<&str> {
        maybe_str_ref(self.region_name.as_ref())
    }

    pub fn role_arn(&self) -> Option<&str> {
        maybe_str_ref(self.role_arn.as_ref())
    }

    pub fn role_session_name(&self) -> Option<&str> {
        maybe_str_ref(self.role_session_name.as_ref())
    }

    pub fn external_id(&self) -> Option<&str> {
        maybe_str_ref(self.external_id.as_ref())
    }

    pub fn duration_seconds(&self) -> Option<u32> {
        self.duration_seconds
    }

    pub fn scope_down_policy(&self) -> Option<&str> {
        maybe_str_ref(self.scope_down_policy.as_ref())
    }

    pub fn mfa_serial(&self) -> Option<&str> {
        maybe_str_ref(self.mfa_serial.as_ref())
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
