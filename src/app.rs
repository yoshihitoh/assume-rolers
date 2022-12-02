use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::Path;

use assume_rolers_schema::credentials::ProfileCredentials;
use async_trait::async_trait;
use clap::builder::{PossibleValue, TypedValueParser};
use clap::ArgAction;

use crate::assume_role::rusoto::RusotoAssumeRole;
use crate::command::federation::FederationCommand;
use crate::command::shell::ShellCommand;
use crate::command::wasm::WasmCommand;
use crate::command::Command;
use crate::mfa::{ReadMfaToken, StaticMfaTokenReader, StdinMfaTokenReader};
use crate::profile::load::aws_sdk::AwsSdkProfileLoader;
use crate::profile::load::LoadProfiles;
use crate::profile::select::skim::SkimProfileSelector;
use crate::profile::select::{SelectProfile, StaticProfileSelector};
use crate::profile::{Profile, ProfileSet};
use crate::run::AssumeRolers;

fn builtin_commands() -> HashMap<&'static str, CredentialsCommand> {
    fn wasm_command(name: &str, binary: Vec<u8>) -> CredentialsCommand {
        CredentialsCommand::WasmPlugin(WasmCommand::from_binary(name, binary))
    }

    HashMap::from([
        (
            "export",
            wasm_command(
                "export",
                include_bytes!("../assets/assume-rolers-export.wasm").to_vec(),
            ),
        ),
        (
            "federation",
            CredentialsCommand::Federation(FederationCommand),
        ),
    ])
}

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

enum CredentialsCommand {
    Shell(ShellCommand),
    WasmPlugin(WasmCommand),
    Federation(FederationCommand),
}

#[async_trait]
impl Command for CredentialsCommand {
    async fn run(self, credentials: ProfileCredentials) -> anyhow::Result<()> {
        use CredentialsCommand::*;
        match self {
            Shell(command) => command.run(credentials).await,
            WasmPlugin(command) => command.run(credentials).await,
            Federation(command) => command.run(credentials).await,
        }
    }
}

fn credentials_command_from(assume_role: &AssumeRole) -> anyhow::Result<CredentialsCommand> {
    if let Some(plugin) = assume_role.plugin.as_ref() {
        let file_ext = Path::new(plugin).extension().and_then(|s| s.to_str());
        let mut commands = builtin_commands();
        if let Some("wasm") = file_ext {
            Ok(CredentialsCommand::WasmPlugin(WasmCommand::from_file(
                plugin,
            )))
        } else if let Some(command) = commands.remove(plugin.as_str()) {
            Ok(command)
        } else {
            Err(anyhow::anyhow!(
                "plugin must be a path to .wasm file, or built-in plugin name."
            ))
        }
    } else {
        Ok(CredentialsCommand::Shell(ShellCommand))
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
                .short('t')
                .long("token")
                .value_hint(clap::ValueHint::Other)
                .help("Specify a token code provided by the MFA device."),
        )
        .arg(
            clap::Arg::new("plugin")
                .short('p')
                .long("plugin")
                .value_hint(clap::ValueHint::FilePath)
                .help("Specify a builtin plugin name, or path to the WebAssembly/WASI file."),
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
    plugin: Option<String>,
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
            let plugin = matches.get_one::<String>("plugin").map(|s| s.to_string());
            App::AssumeRole(AssumeRole {
                profile,
                token,
                plugin,
            })
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
        let command = credentials_command_from(&assume_role)?;
        let assume_rolers = AssumeRolers::new(
            AwsSdkProfileLoader::default(),
            selector,
            mfa_reader,
            RusotoAssumeRole,
            command,
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
