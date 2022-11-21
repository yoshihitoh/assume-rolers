use tracing::error;

use assume_rolers::app::{self, App};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let cmd = app::app().await?;
    let app = App::from(cmd);
    match app.run().await {
        Ok(_) => Ok(()), // never
        Err(e) => {
            error!("error:{:?}", e);
            Err(e)
        }
    }
}
