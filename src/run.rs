use crate::assume_role::AssumeRole;
use crate::command::Command;
use crate::mfa::ReadMfaToken;
use crate::profile::load::LoadProfiles;
use crate::profile::select::SelectProfile;
use assume_rolers_schema::credentials::ProfileCredentials;
use tracing::debug;

pub struct AssumeRolers<L, S, R, A, C> {
    loader: L,
    selector: S,
    mfa_reader: R,
    assume_role: A,
    command: C,
}

impl<L, S, R, A, C> AssumeRolers<L, S, R, A, C>
where
    L: LoadProfiles + Send + Sync + 'static,
    S: SelectProfile,
    R: ReadMfaToken + Send + Sync + 'static,
    A: AssumeRole + Send + Sync + 'static,
    C: Command,
{
    pub fn new(loader: L, selector: S, mfa_reader: R, assume_role: A, command: C) -> Self {
        Self {
            loader,
            selector,
            mfa_reader,
            assume_role,
            command,
        }
    }

    pub async fn run(self) -> anyhow::Result<()> {
        let profiles = self.loader.load_profiles().await?;
        if let Some(profile) = self.selector.select_profile(&profiles)? {
            debug!("target profile:{}", profile.name);
            let result = self
                .assume_role
                .assume_role(profile, self.mfa_reader)
                .await?;

            self.command
                .run(ProfileCredentials {
                    profile_name: profile.name().to_string(),
                    region_name: result.region_name,
                    credentials: result.credentials,
                })
                .await?;
        } else {
            debug!("no profile selected.")
        }

        Ok(())
    }
}
