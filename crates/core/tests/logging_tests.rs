use tracing::{info, Level};
use tracing_subscriber::{fmt, EnvFilter};

#[test]
fn initialize_logging_with_env_filter() {
    // It should be safe to initialize multiple times in tests; ignore errors
    let _ = fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init();

    info!("logging initialized");
    // No assertions on output; just ensure no panic and level type is available
    let _level = Level::INFO;
}
