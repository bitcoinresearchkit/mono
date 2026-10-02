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
