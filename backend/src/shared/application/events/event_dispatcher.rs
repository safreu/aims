use crate::shared::application::events::AppEvent;

pub trait EventDispatcher: Send + Sync {
    fn dispatch(&self, event: AppEvent);
}
