pub use crate::port::event_publisher::*;

pub struct NullEventPublisher;

impl EventPublisher for NullEventPublisher {
    async fn publish(&self, _event: Event) -> PublishResult {
        Ok(())
    }
}
