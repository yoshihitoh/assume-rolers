use std::env;
use std::ffi::CString;
use std::io::{stdin, stdout, Cursor, Write};
use std::str::FromStr;

use aws_config::profile::profile_file::ProfileFiles;
use aws_config::profile::{load, Profile, ProfileSet};
use aws_types::os_shim_internal::{Env, Fs};
use chrono::Duration;
use rusoto_core::credential::AwsCredentials;
use rusoto_core::Region;
use rusoto_sts::{StsAssumeRoleSessionCredentialsProvider, StsClient};
use skim::prelude::*;
use tracing::{debug, error};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    match run().await {
        Ok(_) => Ok(()), // never
        Err(e) => {
            error!("error:{:?}", e);
            Err(e)
        }
    }
}

async fn run() -> anyhow::Result<()> {
    let profiles = load_profiles().await?;

    if let Some(profile_name) = select_profile(&profiles).await? {
        let (region, creds) = assume_role(profiles.get_profile(&profile_name).unwrap()).await?;
        handle_credentials(&profile_name, region, creds)?;
    } else {
        debug!("no profile selected.")
    }

    Ok(())
}

async fn load_profiles() -> anyhow::Result<ProfileSet> {
    let profile_files = ProfileFiles::default();
    let fs = Fs::default();
    let env = Env::default();
    let profiles = load(&fs, &env, &profile_files).await?;

    debug!(
        "profile names: [{}]",
        profiles.profiles().collect::<Vec<_>>().join(", ")
    );
    Ok(profiles)
}

async fn select_profile(profiles: &ProfileSet) -> anyhow::Result<Option<String>> {
    let mut names = profiles
        .profiles()
        .filter(|&p| profiles.get_profile(p).unwrap().get("role_arn").is_some())
        .collect::<Vec<_>>();
    names.sort();
    let item_reader = SkimItemReader::default();
    let items = item_reader.of_bufread(Cursor::new(names.join("\n")));

    let options = SkimOptionsBuilder::default().reverse(true).build()?;
    let selected = Skim::run_with(&options, Some(items))
        .and_then(|out| (!out.is_abort).then(move || out.selected_items))
        .unwrap_or_else(Vec::default);

    Ok(selected.into_iter().next().map(|x| x.output().to_string()))
}

async fn assume_role(profile: &Profile) -> anyhow::Result<(Region, AwsCredentials)> {
    debug!("target profile:{}", profile.name());

    let source_profile = profile.get("source_profile").unwrap_or("default");
    env::set_var("AWS_PROFILE", source_profile);

    let region = profile
        .get("region")
        .map(Region::from_str)
        .unwrap_or(Ok(Region::UsWest1))?;
    let sts_client = StsClient::new(region.clone());

    let role_arn = profile
        .get("role_arn")
        .map(Ok)
        .unwrap_or_else(|| {
            Err(anyhow::anyhow!(
                "no role_arn found. profile:{}",
                profile.name()
            ))
        })?
        .to_string();

    let session_name = profile
        .get("role_session_name")
        .unwrap_or("assume-rolers-cli")
        .to_string();

    let external_id = profile.get("external_id").map(|s| s.to_string());

    let duration_seconds = profile
        .get("duration_seconds")
        .map(|s| s.parse::<i64>().map(Some))
        .unwrap_or(Ok(None))?;
    let session_duration = duration_seconds.map(Duration::seconds);
    let scope_down_policy = profile.get("scope_down_policy").map(|s| s.to_string());
    let mfa_serial = profile.get("mfa_serial").map(|s| s.to_string());

    let mut provider = StsAssumeRoleSessionCredentialsProvider::new(
        sts_client,
        role_arn,
        session_name,
        external_id,
        session_duration,
        scope_down_policy,
        mfa_serial.clone(),
    );

    if mfa_serial.is_some() {
        print!("Enter MFA code for {}: ", mfa_serial.unwrap());
        stdout().flush()?;

        let mut code = String::new();
        stdin().read_line(&mut code)?;
        provider.set_mfa_code(code.trim());
    }

    let creds = provider.assume_role().await?;
    Ok((region, creds))
}

struct Variable<'a> {
    name: &'a str,
    value: Option<&'a str>,
}

fn handle_credentials(profile: &str, region: Region, creds: AwsCredentials) -> anyhow::Result<()> {
    fn v<'a>(name: &'a str, value: Option<&'a str>) -> Variable<'a> {
        Variable { name, value }
    }

    let token = creds.token();
    let variables = [
        // for AWS SDK, aws-cli
        v("AWS_PROFILE", None),
        v("AWS_REGION", Some(region.name())),
        v("AWS_DEFAULT_REGION", Some(region.name())),
        v("AWS_ACCESS_KEY_ID", Some(creds.aws_access_key_id())),
        v("AWS_SECRET_ACCESS_KEY", Some(creds.aws_secret_access_key())),
        v("AWS_SESSION_TOKEN", token.as_ref().map(|s| s.as_str())),
        // for prompts
        v("ASSUME_ROLERS_PROFILE", Some(profile)),
    ];

    for Variable { name, value } in variables {
        if let Some(value) = value {
            env::set_var(name, value);
        } else {
            env::remove_var(name);
        }
    }

    let shell = env::var("SHELL")?;
    debug!("shell: {}, ", &shell);

    let shell = CString::new(shell.bytes().collect::<Vec<_>>())?;
    let args: Vec<CString> = Vec::new();
    nix::unistd::execv(&shell, &args)?;

    // never
    Ok(())
}
