use crate::entity::Meme;
use crate::types::{ByteSize, MemeCaption, MemeId, MemePath, Timestamp};
use ::identity::UserId;

#[derive(Debug, ::thiserror::Error)]
#[error("Failed to publish {event:?}: {source}")]
pub struct PublishError {
    pub source: Box<dyn std::error::Error + Send + Sync>,
    pub event: Event,
}

pub type PublishResult = Result<(), PublishError>;

#[cfg_attr(any(test, feature = "testing"), ::mockall::automock)]
pub trait EventPublisher: Send + Sync {
    fn publish(&self, event: Event) -> impl Future<Output = PublishResult> + Send;

    fn meme_created(
        &self,
        meme: &Meme,
    ) -> impl Future<Output = PublishResult> + Send {
        async move {
            self.publish(Event {
                timestamp: meme.id.created_at(),
                message: Message::MemeCreated {
                    meme_id: meme.id,
                    ower_id: meme.owner_id,
                    file_size: meme.file_size,
                    caption: meme.caption.clone(),
                    path: meme.path.clone(),
                },
            })
            .await
        }
    }

    fn meme_deleted(
        &self,
        meme_id: MemeId,
    ) -> impl Future<Output = PublishResult> + Send {
        async move {
            self.publish(Event {
                timestamp: Timestamp::now(),
                message: Message::MemeDeleted { meme_id },
            })
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
    MemeCreated {
        meme_id: MemeId,
        ower_id: UserId,
        file_size: ByteSize,
        caption: Option<MemeCaption>,
        path: MemePath,
    },
    MemeDeleted {
        meme_id: MemeId,
    },
}
