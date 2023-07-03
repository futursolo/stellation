#![deny(clippy::all)]
#![deny(missing_debug_implementations)]

mod app;
use app::App;
use example_fullstack_api::create_routine_registry;
use stellation_bridge::links::FetchLink;
use stellation_bridge::Bridge;
use tracing_subscriber::filter::LevelFilter;

fn main() {
    stellation_frontend::trace::init_default(LevelFilter::INFO);

    // Create bridge
    let bridge = Bridge::new(
        FetchLink::builder()
            .routines(create_routine_registry())
            .build(),
    );

    // Start Application
    stellation_frontend::Renderer::<App>::new()
        .bridge(bridge)
        .render();
}
