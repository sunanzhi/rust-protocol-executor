use tracing_subscriber;

pub fn init() {
    let _ = tracing_subscriber::fmt()
        .with_test_writer()
        .try_init();
}