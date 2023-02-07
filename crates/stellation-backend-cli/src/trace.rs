//! Tracing support.

use std::env;

use console::style;
use stellation_core::dev::StctlMetadata;
use tracing::field::Visit;
use tracing::{Level, Subscriber};
use tracing_subscriber::filter::filter_fn;
use tracing_subscriber::layer::Context;
use tracing_subscriber::prelude::*;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::{EnvFilter, Layer};

/// A layer that emits pretty access logs for stellation servers.
#[derive(Debug, Default)]
pub struct AccessLog {}

/// Returns a layer that emits pretty access logs.
pub fn pretty_access() -> AccessLog {
    AccessLog {}
}

impl<S> Layer<S> for AccessLog
where
    S: Subscriber + for<'lookup> LookupSpan<'lookup>,
{
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: Context<'_, S>) {
        if event.metadata().target() != "stellation_backend::endpoint::trace" {
            return;
        }

        #[derive(Default, Debug)]
        struct Values {
            duration: Option<u128>,
            path: Option<String>,
            method: Option<String>,
            status: Option<u64>,
        }

        impl Visit for Values {
            fn record_u128(&mut self, field: &tracing::field::Field, value: u128) {
                if field.as_ref() == "duration" {
                    self.duration = Some(value);
                }
            }

            fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
                if field.as_ref() == "path" {
                    self.path = Some(value.to_string());
                }
            }

            fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
                if field.as_ref() == "status" {
                    self.status = Some(value);
                }
            }

            fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
                if field.as_ref() == "method" {
                    self.method = Some(format!("{value:?}"));
                }
            }
        }

        let mut values = Values::default();
        event.record(&mut values);

        if let (Some(path), Some(duration), Some(status), Some(method)) =
            (values.path, values.duration, values.status, values.method)
        {
            let duration = Some(duration)
                .and_then(|m| i32::try_from(m).ok())
                .map(f64::from)
                .expect("duration took too long")
                / 100_000.0;

            let status = match status {
                m if m < 200 => style(m).cyan(),
                m if m < 300 => style(m).green(),
                m if m < 400 => style(m).yellow(),
                m => style(m).red(),
            }
            .bold();

            eprintln!("{method:>6} {status} {duration:>8.2}ms {path}");
        }
    }
}

/// Initialise tracing with default settings.
pub fn init_default<S>(var_name: S)
where
    S: Into<String>,
{
    let var_name = var_name.into();
    let env_filter = EnvFilter::builder()
        .with_default_directive(Level::INFO.into())
        .with_env_var(var_name)
        .from_env_lossy();

    match env::var(StctlMetadata::ENV_NAME) {
        Ok(_) => {
            // Register pretty logging if under development server.
            tracing_subscriber::registry()
                .with(pretty_access())
                .with(
                    tracing_subscriber::fmt::layer()
                        .compact()
                        // access logs are processed by the access log layer
                        .with_filter(filter_fn(|metadata| {
                            metadata.target() != "stellation_backend::endpoint::trace"
                        })),
                )
                .with(env_filter)
                .init();
        }
        Err(_) => {
            tracing_subscriber::registry()
                .with(tracing_subscriber::fmt::layer().compact())
                .with(env_filter)
                .init();
        }
    }
}
