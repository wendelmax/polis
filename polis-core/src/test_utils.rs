use std::sync::Once;
use tracing_subscriber;

static INIT: Once = Once::new();

pub fn init_test_logging() {
    INIT.call_once(|| {
        // Try to initialize, but don't panic if it fails
        let _ = tracing_subscriber::fmt()
            .with_env_filter("debug")
            .try_init();
    });
}

pub fn with_test_logging<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter("debug")
        .with_test_writer()
        .finish();

    tracing::subscriber::with_default(subscriber, f)
}
