#![deny(clippy::all)]
#![deny(missing_debug_implementations)]

mod app;
use app::App;
use example_fullstack_api::create_bridge;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::fmt::format::Pretty;
use tracing_subscriber::fmt::time::UtcTime;
use tracing_subscriber::prelude::*;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_web::{performance_layer, MakeConsoleWriter};

fn main() {
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .with_timer(UtcTime::rfc_3339())
        .with_writer(MakeConsoleWriter);
    let perf_layer = performance_layer().with_details_from_fields(Pretty::default());

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(perf_layer)
        .with(LevelFilter::INFO)
        .init();

    // Create bridge
    let bridge = create_bridge();

    // Start Application
    stackable_frontend::Renderer::<App>::new()
        .bridge(bridge)
        .render();
}
