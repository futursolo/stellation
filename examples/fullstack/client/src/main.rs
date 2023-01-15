#![deny(clippy::all)]
#![deny(missing_debug_implementations)]

mod app;
use app::App;
use example_fullstack_api::create_bridge;
use tracing_subscriber::filter::LevelFilter;

fn main() {
    stellation_frontend::trace::init_default(LevelFilter::INFO);

    // Create bridge
    let bridge = create_bridge();

    // Start Application
    stellation_frontend::Renderer::<App>::new()
        .bridge(bridge)
        .render();
}
