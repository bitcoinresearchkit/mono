use std::sync::OnceLock;

use tracing::{Event, Subscriber};
use tracing_subscriber::{Layer, layer::Context};

type Hook = dyn for<'a> Fn(&Event<'a>) + Send + Sync;

static LOG_HOOK: OnceLock<Box<Hook>> = OnceLock::new();

pub struct HookLayer;

impl<S: Subscriber> Layer<S> for HookLayer {
    fn on_event(&self, event: &Event<'_>, _: Context<'_, S>) {
        if let Some(hook) = LOG_HOOK.get() {
            hook(event);
        }
    }
}

pub fn register<F>(hook: F) -> Result<(), &'static str>
where
    F: for<'a> Fn(&Event<'a>) + Send + Sync + 'static,
{
    LOG_HOOK
        .set(Box::new(hook))
        .map_err(|_| "Hook already registered")
}

#[cfg(test)]
mod tests {
    use std::{
        fmt::Debug,
        sync::{
            Arc,
            atomic::{AtomicU64, Ordering},
        },
    };

    use tracing::{
        field::{Field, Visit},
        info, subscriber,
    };
    use tracing_subscriber::{layer::SubscriberExt, registry};

    use super::*;

    struct DurationVisitor<'a>(&'a AtomicU64);

    impl Visit for DurationVisitor<'_> {
        fn record_u64(&mut self, field: &Field, value: u64) {
            if field.name() == "duration_ns" {
                self.0.store(value, Ordering::Relaxed);
            }
        }

        fn record_debug(&mut self, _field: &Field, _value: &dyn Debug) {}
    }

    #[test]
    fn forwards_structured_events() {
        let duration = Arc::new(AtomicU64::new(0));
        let hook_duration = Arc::clone(&duration);
        register(move |event| {
            if event.metadata().target() == "test_timing" {
                event.record(&mut DurationVisitor(&hook_duration));
            }
        })
        .unwrap();

        let subscriber = registry().with(HookLayer);
        subscriber::with_default(subscriber, || {
            info!(target: "test_timing", phase = "compute", duration_ns = 1_u64);
        });

        assert_eq!(duration.load(Ordering::Relaxed), 1);
    }
}
