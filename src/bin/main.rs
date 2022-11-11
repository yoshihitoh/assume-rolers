use async_trait::async_trait;
use clap::{arg, Parser};
use tracing::error;

use assume_rolers::assume_role::rusoto::RusotoAssumeRole;
use assume_rolers::handler::shell::ShellCredentialsHandler;
use assume_rolers::mfa::{ReadMfaToken, StaticMfaTokenReader, StdinMfaTokenReader};
use assume_rolers::profile::load::aws_sdk::AwsSdkProfileLoader;
use assume_rolers::profile::{Profile, ProfileSet};
use assume_rolers::profile::select::skim::SkimProfileSelector;
use assume_rolers::profile::select::{SelectProfile, StaticProfileSelector};
use assume_rolers::run::AssumeRolers;

#[derive(Parser, Debug)]
struct Args {
    #[arg()]
    profile: Option<String>,

    #[arg(short, long)]
    token: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    match run(args).await {
        Ok(_) => Ok(()), // never
        Err(e) => {
            error!("error:{:?}", e);
            Err(e)
        }
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

fn selector_from(args: &Args) -> ProfileSelector {
    if let Some(profile) = args.profile.as_ref() {
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

fn mfa_reader_from(args: &Args) -> MfaReader {
    if let Some(token) = args.token.as_ref() {
        MfaReader::Static(StaticMfaTokenReader::from(token))
    } else {
        MfaReader::Stdin(StdinMfaTokenReader)
    }
}

async fn run(args: Args) -> anyhow::Result<()> {
    let selector = selector_from(&args);
    let mfa_reader = mfa_reader_from(&args);
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
