// use tracing_subscriber::prelude::*;
// use tracing_web::MakeConsoleWriter;

mod app;
use app::App;
use example_fullstack_api::{create_bridge, ServerTimeQuery};
use yew::platform::spawn_local;

fn main() {
    spawn_local(async move {
        let bridge = create_bridge();
        let t = bridge.resolve_query::<ServerTimeQuery>(&()).await;

        gloo::console::log!(format!("{:?}", t.expect("failed to load")));
    });

    // Setup Logging
    // let fmt_layer = tracing_subscriber::fmt::layer()
    //     .with_ansi(false)
    //     .with_writer(MakeConsoleWriter);
    // tracing_subscriber::registry().with(fmt_layer).init();

    // Start Application
    stackable_frontend::Renderer::<App>::new().hydrate();
}
