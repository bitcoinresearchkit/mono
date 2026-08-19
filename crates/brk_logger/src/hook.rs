use std::{fmt::Write, sync::OnceLock};

use tracing::{Event, Subscriber, field::Field};

static LOG_HOOK: OnceLock<Box<dyn Fn(&str) + Send + Sync>> = OnceLock::new();

pub struct HookLayer;

impl<S: Subscriber> tracing_subscriber::Layer<S> for HookLayer {
    fn on_event(&self, event: &Event<'_>, _: tracing_subscriber::layer::Context<'_, S>) {
        if let Some(hook) = LOG_HOOK.get() {
            let mut msg = String::new();
            event.record(&mut MessageVisitor(&mut msg));
            hook(&msg);
        }
    }
}

pub fn register<F>(hook: F) -> Result<(), &'static str>
where
    F: Fn(&str) + Send + Sync + 'static,
{
    LOG_HOOK
        .set(Box::new(hook))
        .map_err(|_| "Hook already registered")
}

struct MessageVisitor<'a>(&'a mut String);

impl tracing::field::Visit for MessageVisitor<'_> {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.0.clear();
            let _ = write!(self.0, "{value:?}");
        }
    }
}
