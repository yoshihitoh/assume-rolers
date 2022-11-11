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
    value: Option<&'a str>,
}

fn into_variables(request: HandleCredentialsRequest) -> Vec<Variable> {
    fn v<'a>(name: &'a str, value: Option<&'a str>) -> Variable<'a> {
        Variable { name, value }
    }

    vec![
        // for AWS SDK, aws-cli
        v("AWS_PROFILE", None),
        v("AWS_REGION", Some(request.region_name)),
        v("AWS_DEFAULT_REGION", Some(request.region_name)),
        v("AWS_ACCESS_KEY_ID", Some(request.credentials.key())),
        v("AWS_SECRET_ACCESS_KEY", Some(request.credentials.secret())),
        v("AWS_SESSION_TOKEN", request.credentials.token()),
        // for prompts
        v("ASSUME_ROLERS_PROFILE", Some(request.profile_name)),
    ]
}
