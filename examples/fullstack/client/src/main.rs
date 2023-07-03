#![deny(clippy::all)]
#![deny(missing_debug_implementations)]

mod app;
use app::App;
use example_fullstack_api::create_frontend_bridge;
use tracing_subscriber::filter::LevelFilter;

fn main() {
    // Configures Logging
    stellation_frontend::trace::init_default(LevelFilter::INFO);

    // Starts Application
    stellation_frontend::Renderer::<App>::new()
        .bridge(create_frontend_bridge())
        .render();
}
