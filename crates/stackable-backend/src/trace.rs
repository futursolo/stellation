use console::style;
use tracing::field::Visit;
use tracing::Subscriber;
use tracing_subscriber::layer::Context;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::Layer;

#[derive(Debug, Default)]
pub struct AccessLog {}

pub fn pretty_access() -> AccessLog {
    AccessLog {}
}

impl<S> Layer<S> for AccessLog
where
    S: Subscriber + for<'lookup> LookupSpan<'lookup>,
{
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: Context<'_, S>) {
        if event.metadata().target() != "stackable_backend::endpoint::trace" {
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
                    self.method = Some(format!("{:?}", value));
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

            eprintln!("{method:>6} {status} {:>8.2}ms {path}", duration);
        }
    }
}
