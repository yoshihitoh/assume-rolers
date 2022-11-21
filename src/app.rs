use std::ffi::OsStr;

use async_trait::async_trait;
use clap::builder::{PossibleValue, TypedValueParser};
use clap::ArgAction;

use crate::assume_role::rusoto::RusotoAssumeRole;
use crate::handler::shell::ShellCredentialsHandler;
use crate::mfa::{ReadMfaToken, StaticMfaTokenReader, StdinMfaTokenReader};
use crate::profile::load::aws_sdk::AwsSdkProfileLoader;
use crate::profile::load::LoadProfiles;
use crate::profile::select::skim::SkimProfileSelector;
use crate::profile::select::{SelectProfile, StaticProfileSelector};
use crate::profile::{Profile, ProfileSet};
use crate::run::AssumeRolers;

async fn profile_names<L: LoadProfiles>(loader: L) -> anyhow::Result<Vec<String>> {
    let profiles = loader.load_profiles().await?;
    Ok(profiles
        .profiles()
        .filter_map(|p| p.role_arn.as_ref().map(|_| p.name.to_string()))
        .collect::<Vec<_>>())
}

#[derive(Debug, Clone)]
struct ProfileNameParser {
    profile_names: Vec<String>,
}

impl From<Vec<String>> for ProfileNameParser {
    fn from(profile_names: Vec<String>) -> Self {
        ProfileNameParser { profile_names }
    }
}

impl TypedValueParser for ProfileNameParser {
    type Value = String;

    fn parse_ref(
        &self,
        cmd: &clap::Command,
        _arg: Option<&clap::Arg>,
        value: &OsStr,
    ) -> Result<Self::Value, clap::Error> {
        let value = value
            .to_os_string()
            .into_string()
            .map_err(|_| clap::Error::new(clap::error::ErrorKind::InvalidUtf8).with_cmd(cmd))?;

        if self.profile_names.contains(&value) {
            Ok(value)
        } else {
            Err(clap::Error::new(clap::error::ErrorKind::InvalidValue).with_cmd(cmd))
        }
    }

    fn possible_values(&self) -> Option<Box<dyn Iterator<Item = PossibleValue> + '_>> {
        Some(Box::new(
            self.profile_names
                .clone()
                .into_iter()
                .map(PossibleValue::new),
        ))
    }
}

enum ProfileSelector {
    Skim(SkimProfileSelector),
    Static(StaticProfileSelector),
}

impl SelectProfile for ProfileSelector {
    fn select_profile<'a>(&self, profiles: &'a ProfileSet) -> anyhow::Result<Option<&'a Profile>> {
        use ProfileSelector::*;
        match self {
            Skim(s) => s.select_profile(profiles),
            Static(s) => s.select_profile(profiles),
        }
    }
}

fn selector_from(assume_role: &AssumeRole) -> ProfileSelector {
    if let Some(profile) = assume_role.profile.as_ref() {
        ProfileSelector::Static(StaticProfileSelector::from(profile.to_string()))
    } else {
        ProfileSelector::Skim(SkimProfileSelector)
    }
}

enum MfaReader {
    Stdin(StdinMfaTokenReader),
    Static(StaticMfaTokenReader),
}

#[async_trait]
impl ReadMfaToken for MfaReader {
    async fn read_mfa_token(&self, mfa_serial: &str) -> anyhow::Result<String> {
        use MfaReader::*;
        match self {
            Stdin(r) => r.read_mfa_token(mfa_serial).await,
            Static(r) => r.read_mfa_token(mfa_serial).await,
        }
    }
}

fn mfa_reader_from(assume_role: &AssumeRole) -> MfaReader {
    if let Some(token) = assume_role.token.as_ref() {
        MfaReader::Static(StaticMfaTokenReader::from(token))
    } else {
        MfaReader::Stdin(StdinMfaTokenReader)
    }
}

pub async fn app() -> anyhow::Result<clap::Command> {
    let profile_names = profile_names(AwsSdkProfileLoader::default()).await?;
    let name_parser = ProfileNameParser::from(profile_names);

    Ok(clap::Command::new("assume-rolers")
        .disable_colored_help(false)
        .arg(
            clap::Arg::new("profile")
                .value_hint(clap::ValueHint::Other)
                .value_parser(name_parser)
                .help("Specify a profile to assume."),
        )
        .arg(
            clap::Arg::new("token")
                .value_hint(clap::ValueHint::Other)
                .help("Specify a token code provided by the MFA device."),
        )
        .arg(
            clap::Arg::new("list")
                .short('l')
                .long("list")
                .action(ArgAction::SetTrue)
                .conflicts_with_all(["profile", "token"])
                .help("Show available profiles."),
        ))
}

#[derive(Debug)]
pub struct AssumeRole {
    profile: Option<String>,
    token: Option<String>,
}

#[derive(Debug)]
pub struct ListProfiles;

#[derive(Debug)]
pub enum App {
    AssumeRole(AssumeRole),
    ListProfiles(ListProfiles),
}

impl From<clap::Command> for App {
    fn from(c: clap::Command) -> Self {
        let matches = c.get_matches();
        if matches.get_flag("list") {
            App::ListProfiles(ListProfiles)
        } else {
            let profile = matches.get_one::<String>("profile").map(|s| s.to_string());
            let token = matches.get_one::<String>("token").map(|s| s.to_string());
            App::AssumeRole(AssumeRole { profile, token })
        }
    }
}

impl App {
    pub async fn run(self) -> anyhow::Result<()> {
        match self {
            App::AssumeRole(assume_role) => Self::assume_role(assume_role).await,
            App::ListProfiles(list_profiles) => Self::list_profiles(list_profiles).await,
        }
    }

    async fn assume_role(assume_role: AssumeRole) -> anyhow::Result<()> {
        let selector = selector_from(&assume_role);
        let mfa_reader = mfa_reader_from(&assume_role);
        let assume_rolers = AssumeRolers::new(
            AwsSdkProfileLoader::default(),
            selector,
            mfa_reader,
            RusotoAssumeRole,
            ShellCredentialsHandler,
        );
        assume_rolers.run().await?;
        Ok(())
    }

    async fn list_profiles(_list_profiles: ListProfiles) -> anyhow::Result<()> {
        let profile_names = profile_names(AwsSdkProfileLoader::default()).await?;
        for p in profile_names {
            println!("{}", p);
        }

        Ok(())
    }
}
