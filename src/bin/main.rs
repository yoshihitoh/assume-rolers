use tracing::error;

use assume_rolers::assume_role::rusoto::RusotoAssumeRole;
use assume_rolers::handler::shell::ShellCredentialsHandler;
use assume_rolers::mfa::{StdinMfaTokenReader};
use assume_rolers::profile::load::aws_sdk::AwsSdkProfileLoader;
use assume_rolers::profile::select::skim::SkimProfileSelector;
use assume_rolers::run::AssumeRolers;

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
    let assume_rolers = AssumeRolers::new(
        AwsSdkProfileLoader::default(),
        SkimProfileSelector,
        StdinMfaTokenReader,
        RusotoAssumeRole,
        ShellCredentialsHandler,
    );
    assume_rolers.run().await?;
    Ok(())
}
