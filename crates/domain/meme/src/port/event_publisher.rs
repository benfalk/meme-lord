use crate::entity::{Meme, UserTag, UserTagLink};
use crate::types::{
    ByteSize, MemeCaption, MemeId, MemePath, TagId, TagName, Timestamp,
};
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

    fn user_tag_created(
        &self,
        tag: &UserTag,
    ) -> impl Future<Output = PublishResult> + Send {
        async move {
            self.publish(Event {
                timestamp: tag.id.created_at(),
                message: Message::UserTagCreated {
                    tag_id: tag.id,
                    owner_id: tag.owner_id,
                    name: tag.name.clone(),
                },
            })
            .await
        }
    }

    fn user_tag_updated(
        &self,
        tag: &UserTag,
    ) -> impl Future<Output = PublishResult> + Send {
        async move {
            self.publish(Event::new(Message::UserTagUpdated {
                tag_id: tag.id,
                owner_id: tag.owner_id,
                name: tag.name.clone(),
            }))
            .await
        }
    }

    fn user_tag_deleted(
        &self,
        tag_id: TagId,
    ) -> impl Future<Output = PublishResult> + Send {
        async move {
            self.publish(Event::new(Message::UserTagDeleted { tag_id }))
                .await
        }
    }

    fn user_tag_link_created(
        &self,
        link: &UserTagLink,
    ) -> impl Future<Output = PublishResult> + Send {
        async move {
            self.publish(Event::new(Message::UserTagLinkCreated {
                tag_id: link.tag_id,
                meme_id: link.meme_id,
            }))
            .await
        }
    }

    fn user_tag_link_deleted(
        &self,
        link: &UserTagLink,
    ) -> impl Future<Output = PublishResult> + Send {
        async move {
            self.publish(Event::new(Message::UserTagLinkDeleted {
                tag_id: link.tag_id,
                meme_id: link.meme_id,
            }))
            .await
        }
    }
}

#[derive(Debug, Clone)]
pub struct Event {
    pub timestamp: Timestamp,
    pub message: Message,
}

impl Event {
    pub fn new(message: Message) -> Self {
        Self {
            timestamp: Timestamp::now(),
            message,
        }
    }
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
    UserTagCreated {
        tag_id: TagId,
        owner_id: UserId,
        name: TagName,
    },
    UserTagUpdated {
        tag_id: TagId,
        owner_id: UserId,
        name: TagName,
    },
    UserTagDeleted {
        tag_id: TagId,
    },
    UserTagLinkCreated {
        tag_id: TagId,
        meme_id: MemeId,
    },
    UserTagLinkDeleted {
        tag_id: TagId,
        meme_id: MemeId,
    },
}
