use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer;

pub fn init_tracer(verbose: bool) {
    let filter = tracing_subscriber::filter::filter_fn(move |metadata| {
        if let Some(x) = metadata.module_path() {
            if !x.starts_with(std::env!("CARGO_CRATE_NAME")) {
                return false;
            }
        }
        let level = match verbose {
            true => tracing::Level::INFO,
            false => tracing::Level::WARN,
        };
        *metadata.level() <= level
    });

    let layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .without_time()
        .with_level(false);

    tracing_subscriber::registry()
        .with(layer.with_filter(filter))
        .init();
}
