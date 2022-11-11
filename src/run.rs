use crate::assume_role::AssumeRole;
use crate::handler::{HandleCredentials, HandleCredentialsRequest};
use crate::mfa::ReadMfaToken;
use crate::profile::load::LoadProfiles;
use crate::profile::select::SelectProfile;
use tracing::debug;

pub struct AssumeRolers<L, S, R, A, H> {
    loader: L,
    selector: S,
    mfa_reader: R,
    assume_role: A,
    handler: H,
}

impl<L, S, R, A, H> AssumeRolers<L, S, R, A, H>
where
    L: LoadProfiles + Send + Sync + 'static,
    S: SelectProfile,
    R: ReadMfaToken + Send + Sync + 'static,
    A: AssumeRole + Send + Sync + 'static,
    H: HandleCredentials,
{
    pub fn new(loader: L, selector: S, mfa_reader: R, assume_role: A, handler: H) -> Self {
        Self {
            loader,
            selector,
            mfa_reader,
            assume_role,
            handler,
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

            self.handler.handle_credentials(HandleCredentialsRequest {
                profile_name: profile.name(),
                region_name: &result.region_name,
                credentials: &result.credentials,
            })?;
        } else {
            debug!("no profile selected.")
        }

        Ok(())
    }
}
