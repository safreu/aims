use crate::shared::application::events::AppEvent;

#[derive(Debug)]
pub struct ApplicationResult<T> {
    value: T,
    events: Vec<AppEvent>,
}

impl<T> ApplicationResult<T> {
    pub fn new(value: T) -> Self {
        Self {
            value,
            events: Vec::new(),
        }
    }

    pub fn with_event(mut self, event: impl Into<AppEvent>) -> Self {
        self.events.push(event.into());
        self
    }

    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn into_parts(self) -> (T, Vec<AppEvent>) {
        (self.value, self.events)
    }
}
