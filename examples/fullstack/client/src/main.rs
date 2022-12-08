// use tracing_subscriber::prelude::*;
// use tracing_web::MakeConsoleWriter;

mod app;
use app::App;
use example_fullstack_api::create_bridge;

fn main() {
    // Setup Logging
    // let fmt_layer = tracing_subscriber::fmt::layer()
    //     .with_ansi(false)
    //     .with_writer(MakeConsoleWriter);
    // tracing_subscriber::registry().with(fmt_layer).init();

    // Start Application
    let bridge = create_bridge();
    stackable_frontend::Renderer::<App>::new()
        .bridge(bridge)
        .hydrate();
}
