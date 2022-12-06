use example_fullstack_app::App;
// use tracing_subscriber::prelude::*;
// use tracing_web::MakeConsoleWriter;

mod app;

fn main() {
    // Setup Logging
    // let fmt_layer = tracing_subscriber::fmt::layer()
    //     .with_ansi(false)
    //     .with_writer(MakeConsoleWriter);
    // tracing_subscriber::registry().with(fmt_layer).init();

    // Start Application
    stackable_frontend::Renderer::<App>::new().hydrate();
}
