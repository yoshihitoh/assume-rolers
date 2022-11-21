use crate::credentials::Credentials;

pub mod shell;

pub struct HandleCredentialsRequest<'a> {
    pub profile_name: &'a str,
    pub region_name: &'a str,
    pub credentials: &'a Credentials,
}

pub trait HandleCredentials {
    fn handle_credentials(&self, request: HandleCredentialsRequest) -> anyhow::Result<()>;
}

struct Variable<'a> {
    name: &'a str,
    value: Option<String>,
}

fn into_variables(request: HandleCredentialsRequest) -> Vec<Variable> {
    fn v<S: Into<String>>(name: &str, value: Option<S>) -> Variable {
        Variable {
            name,
            value: value.map(|s| s.into()),
        }
    }

    vec![
        // for AWS SDK, aws-cli
        v("AWS_PROFILE", Option::<String>::None),
        v("AWS_REGION", Some(request.region_name)),
        v("AWS_DEFAULT_REGION", Some(request.region_name)),
        v("AWS_ACCESS_KEY_ID", Some(request.credentials.key())),
        v("AWS_SECRET_ACCESS_KEY", Some(request.credentials.secret())),
        v("AWS_SESSION_TOKEN", request.credentials.token()),
        v(
            "AWS_SESSION_EXPIRATION",
            request.credentials.expires_at.map(|dt| dt.to_rfc3339()),
        ),
        // for prompts
        v("ASSUME_ROLERS_PROFILE", Some(request.profile_name)),
    ]
}
