//! Request semantics shared by deadlines and response caching. A handler can mark a safe
//! HTTP method as an action before dispatch, as a GET transaction submission must.

use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

use axum::http::Method;

#[derive(Clone)]
pub(crate) struct RequestAction(Arc<AtomicBool>);

impl RequestAction {
    pub fn new(method: &Method) -> Self {
        Self(Arc::new(AtomicBool::new(!method.is_safe())))
    }

    #[cfg(feature = "full-api")]
    pub fn mark(&self) {
        self.0.store(true, Ordering::Relaxed);
    }

    pub fn is_action(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }
}

#[cfg(all(test, feature = "full-api"))]
mod tests {
    use super::*;

    #[test]
    fn action_mark_survives_dispatch_without_leaking_to_another_request() {
        let request = RequestAction::new(&Method::GET);
        let observer = request.clone();
        let unrelated = RequestAction::new(&Method::GET);
        assert!(!observer.is_action());
        request.mark();
        drop(request);
        assert!(observer.is_action());
        assert!(!unrelated.is_action());
        assert!(RequestAction::new(&Method::POST).is_action());
        assert!(!RequestAction::new(&Method::HEAD).is_action());
    }
}
