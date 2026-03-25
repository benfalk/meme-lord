use crate::port::event_publisher::*;

#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct NullEventPublisher;

impl EventPublisher for NullEventPublisher {
    async fn publish(&self, _event: Event) -> PublishResult {
        Ok(())
    }
}
