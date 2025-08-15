use std::env::var;

pub fn init_tracing() {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_env_filter(tracing_subscriber::EnvFilter::new(
            var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .init();
}