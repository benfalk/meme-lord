use crate::types::{Timestamp, UserId};

#[derive(Debug, ::thiserror::Error)]
#[error("Failed to publish event: {source}")]
pub struct PublishError {
    pub source: Box<dyn std::error::Error + Send + Sync>,
    pub event: Event,
}

type PublishSuccess = Result<(), PublishError>;

#[cfg_attr(any(test, feature = "testing"), ::mockall::automock)]
pub trait EventPublisher: Send + Sync {
    fn publish(&self, event: Event) -> impl Future<Output = PublishSuccess> + Send;

    fn user_created(
        &self,
        user_id: UserId,
    ) -> impl Future<Output = PublishSuccess> + Send {
        async move { self.publish(Event::UserCreated { user_id }).await }
    }

    fn user_signed_in(
        &self,
        user_id: UserId,
    ) -> impl Future<Output = PublishSuccess> + Send {
        async move {
            let timestamp = Timestamp::now();
            self.publish(Event::UserSignedIn { user_id, timestamp })
                .await
        }
    }

    fn user_password_changed(
        &self,
        user_id: UserId,
    ) -> impl Future<Output = PublishSuccess> + Send {
        async move {
            let timestamp = Timestamp::now();
            self.publish(Event::UserPasswordChanged { user_id, timestamp })
                .await
        }
    }
}

#[derive(Debug, Clone)]
pub enum Event {
    UserCreated {
        user_id: UserId,
    },
    UserSignedIn {
        user_id: UserId,
        timestamp: Timestamp,
    },
    UserPasswordChanged {
        user_id: UserId,
        timestamp: Timestamp,
    },
}
