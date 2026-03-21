use crate::types::{Timestamp, UserId};

#[derive(Debug, ::thiserror::Error)]
#[error("Failed to publish event: {source}")]
pub struct PublishError {
    pub source: Box<dyn std::error::Error + Send + Sync>,
    pub event: Event,
}

pub type PublishResult = Result<(), PublishError>;

#[cfg_attr(any(test, feature = "testing"), ::mockall::automock)]
pub trait EventPublisher: Send + Sync {
    fn publish(&self, event: Event) -> impl Future<Output = PublishResult> + Send;

    fn user_created(
        &self,
        user_id: UserId,
    ) -> impl Future<Output = PublishResult> + Send {
        async move {
            self.publish(Event {
                timestamp: user_id.created_at(),
                message: Message::UserCreated { user_id },
            })
            .await
        }
    }

    fn user_signed_in(
        &self,
        user_id: UserId,
    ) -> impl Future<Output = PublishResult> + Send {
        async move {
            self.publish(Event::new(Message::UserSignedIn { user_id }))
                .await
        }
    }

    fn user_password_changed(
        &self,
        user_id: UserId,
    ) -> impl Future<Output = PublishResult> + Send {
        async move {
            self.publish(Event::new(Message::UserPasswordChanged { user_id }))
                .await
        }
    }
}

#[derive(Debug, Clone)]
pub struct Event {
    pub timestamp: Timestamp,
    pub message: Message,
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum Message {
    UserCreated { user_id: UserId },
    UserSignedIn { user_id: UserId },
    UserPasswordChanged { user_id: UserId },
}

impl Event {
    pub fn new(message: Message) -> Self {
        Self {
            timestamp: Timestamp::now(),
            message,
        }
    }
}
