use anyhow::Result;
use user_service::bootstrap::app::setup::run_app;

#[tokio::main]
async fn main() -> Result<()> {
    run_app().await
}
