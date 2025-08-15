use anyhow::{Context, Result};
use tokio::signal;

pub async fn wait_shutdown_signal() -> Result<()> {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .context("Failed to install Ctrl+C controller")
    };

    #[cfg(unix)]
    let terminate = async {
        match signal::unix::signal(signal::unix::SignalKind::terminate()) {
            Ok(mut stream) => {
                stream.recv().await;
                Ok(())
            }
            Err(e) => Err(anyhow::Error::new(e).context("Failed to install SIGTERM controller")),
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending();

    tokio::select! {
        res = ctrl_c => res?,
        res = terminate => res?,
    }

    Ok(())
}