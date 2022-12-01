use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::Path;

use assume_rolers_schema::credentials::ProfileCredentials;
use async_trait::async_trait;
use clap::builder::{PossibleValue, TypedValueParser};
use clap::ArgAction;

use crate::assume_role::rusoto::RusotoAssumeRole;
use crate::handler::federation::FederationHandler;
use crate::handler::shell::ShellHandler;
use crate::handler::wasm::WasmHandler;
use crate::handler::HandleCredentials;
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

enum CredentialsHandler {
    Shell(ShellHandler),
    WasmPlugin(WasmHandler),
    Federation(FederationHandler),
}

#[async_trait]
impl HandleCredentials for CredentialsHandler {
    async fn handle_credentials(self, credentials: ProfileCredentials) -> anyhow::Result<()> {
        use CredentialsHandler::*;
        match self {
            Shell(handler) => handler.handle_credentials(credentials).await,
            WasmPlugin(handler) => handler.handle_credentials(credentials).await,
            Federation(handler) => handler.handle_credentials(credentials).await,
        }
    }
}

fn credentials_handler_from(assume_role: &AssumeRole) -> anyhow::Result<CredentialsHandler> {
    if let Some(plugin) = assume_role.plugin.as_ref() {
        let mut builtin_plugins: HashMap<&'static str, Vec<u8>> = vec![(
            "export",
            include_bytes!("../plugins/assume-rolers-export.wasm").to_vec(),
        )]
        .into_iter()
        .collect();
        let file_ext = Path::new(plugin).extension().and_then(|s| s.to_str());
        if let Some("wasm") = file_ext {
            Ok(CredentialsHandler::WasmPlugin(WasmHandler::from_file(
                plugin,
            )))
        } else if let Some(binary) = builtin_plugins.remove(plugin.as_str()) {
            Ok(CredentialsHandler::WasmPlugin(WasmHandler::from_binary(
                plugin, binary,
            )))
        } else if plugin == "federation" {
            Ok(CredentialsHandler::Federation(FederationHandler))
        } else {
            Err(anyhow::anyhow!(
                "plugin must be a path to .wasm file, or built-in plugin name."
            ))
        }
    } else {
        Ok(CredentialsHandler::Shell(ShellHandler))
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
        let handler = credentials_handler_from(&assume_role)?;
        let assume_rolers = AssumeRolers::new(
            AwsSdkProfileLoader::default(),
            selector,
            mfa_reader,
            RusotoAssumeRole,
            handler,
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
