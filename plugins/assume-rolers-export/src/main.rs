use assume_rolers_schema::credentials::ProfileCredentials;
use assume_rolers_schema::plugin::PluginPayload;
use assume_rolers_schema::shell::Shell;

struct EnvironmentVariable<'a> {
    name: &'a str,
    value: Option<String>,
}

fn into_variables(credentials: &ProfileCredentials) -> Vec<EnvironmentVariable> {
    fn v<S: Into<String>>(name: &str, value: Option<S>) -> EnvironmentVariable {
        let value = value.map(|s| s.into());
        EnvironmentVariable { name, value }
    }

    vec![
        // for AWS SDK, aws-cli
        v("AWS_PROFILE", Option::<String>::None),
        v("AWS_REGION", Some(credentials.region_name.as_str())),
        v("AWS_DEFAULT_REGION", Some(credentials.region_name.as_str())),
        v("AWS_ACCESS_KEY_ID", Some(credentials.credentials.key())),
        v(
            "AWS_SECRET_ACCESS_KEY",
            Some(credentials.credentials.secret()),
        ),
        v("AWS_SESSION_TOKEN", credentials.credentials.token()),
        v(
            "AWS_SESSION_EXPIRATION",
            credentials.credentials.expires_at.map(|dt| dt.to_rfc3339()),
        ),
        // for prompts
        v(
            "ASSUME_ROLERS_PROFILE",
            Some(credentials.profile_name.as_str()),
        ),
    ]
}

fn main() -> anyhow::Result<()> {
    let payload = PluginPayload::from_stdin()?;
    if let Some(shell) = payload.shell.as_ref() {
        match shell {
            Shell::Bash => export_bash(&payload),
            Shell::Zsh => export_zsh(&payload),
            Shell::Fish => export_fish(&payload),
            Shell::Unknown(s) => Err(anyhow::anyhow!("unsupported shell. shell:{}", s))?,
        }
    }

    Ok(())
}

fn handle_variables<FSet, FUnset>(payload: &PluginPayload, set: FSet, unset: FUnset)
where
    FSet: Fn(&str, String),
    FUnset: Fn(&str),
{
    for env_var in into_variables(&payload.credentials) {
        if let Some(v) = env_var.value {
            set(env_var.name, v)
        } else {
            unset(env_var.name)
        }
    }
}

fn export_bash(payload: &PluginPayload) {
    handle_variables(
        payload,
        |k, v| {
            println!(r#"export "{}={}""#, k, v);
        },
        |k| {
            println!("unset {}", k);
        },
    );
}

fn export_zsh(payload: &PluginPayload) {
    handle_variables(
        payload,
        |k, v| {
            println!(r#"export "{}={}""#, k, v);
        },
        |k| {
            println!("unset {}", k);
        },
    );
}

fn export_fish(payload: &PluginPayload) {
    handle_variables(
        payload,
        |k, v| {
            println!(r#"set -gx {} "{}""#, k, v);
        },
        |k| {
            println!("set -e {}", k);
        },
    );
}
