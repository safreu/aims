use std::sync::Arc;

use crate::shared::application::events::{ApplicationResult, EventDispatcher};

pub struct ApplicationResultDispatcher {
    event_dispatcher: Arc<dyn EventDispatcher>,
}

impl ApplicationResultDispatcher {
    pub fn new(event_dispatcher: Arc<dyn EventDispatcher>) -> Self {
        Self { event_dispatcher }
    }

    pub fn dispatch<T>(&self, result: ApplicationResult<T>) -> T {
        let (value, events) = result.into_parts();

        for event in events {
            self.event_dispatcher.dispatch(event);
        }

        value
    }
}
