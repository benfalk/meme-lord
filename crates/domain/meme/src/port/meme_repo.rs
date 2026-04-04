use crate::entity::{Meme, UserTag, UserTagLink};
use crate::types::{MemeId, MemePath, TagId, TagName};
use ::identity::UserId;

#[cfg_attr(any(test, feature = "testing"), ::mockall::automock)]
pub trait MemeRepo: Send + Sync {
    fn insert_meme(
        &self,
        meme: &Meme,
    ) -> impl Future<Output = Result<(), InsertMemeError>> + Send;

    fn delete_meme(
        &self,
        id: &MemeId,
    ) -> impl Future<Output = Result<(), DeleteByIdMemeError>> + Send;

    fn update_meme_by_id(
        &self,
        meme: &Meme,
    ) -> impl Future<Output = Result<(), UpdateByIdMemeError>> + Send;

    fn fetch_meme_by_id(
        &self,
        id: &MemeId,
    ) -> impl Future<Output = Result<Meme, FetchMemeByIdError>> + Send;

    fn insert_user_tag(
        &self,
        tag: &UserTag,
    ) -> impl Future<Output = Result<(), InsertUserTagError>> + Send;

    fn update_user_tag_by_id(
        &self,
        tag: &UserTag,
    ) -> impl Future<Output = Result<(), UpdateUserTagByIdError>> + Send;

    fn delete_user_tag_by_id(
        &self,
        id: &TagId,
    ) -> impl Future<Output = Result<(), DeleteUserTagByIdError>> + Send;

    fn insert_user_tag_link(
        &self,
        tag_link: &UserTagLink,
    ) -> impl Future<Output = Result<(), InsertUserTagLinkError>> + Send;

    fn delete_user_tag_link(
        &self,
        tag_link: &UserTagLink,
    ) -> impl Future<Output = Result<(), DeleteUserTagLinkError>> + Send;

    fn user_tags(
        &self,
        owner_id: &UserId,
    ) -> impl Future<Output = Result<Vec<UserTag>, UserTagsError>> + Send;
}

#[derive(Debug, ::thiserror::Error)]
#[error(transparent)]
pub enum MemeRepoError {
    InsertMeme(#[from] InsertMemeError),
    DeleteByIdMeme(#[from] DeleteByIdMemeError),
    UpdateByIdMeme(#[from] UpdateByIdMemeError),
    FetchMemeById(#[from] FetchMemeByIdError),
    InsertUserTag(#[from] InsertUserTagError),
    InsertUserTagLink(#[from] InsertUserTagLinkError),
    UpdateUserTagById(#[from] UpdateUserTagByIdError),
    DeleteUserTagById(#[from] DeleteUserTagByIdError),
    DeleteUserTagLink(#[from] DeleteUserTagLinkError),
    UserTags(#[from] UserTagsError),
}

#[derive(Debug, ::thiserror::Error)]
pub enum InsertMemeError {
    #[error("insert-meme already-exists: {id}")]
    MemeAlreadyExists { id: MemeId },
    #[error("insert-meme path-taken: {path}")]
    PathTaken { path: MemePath },
    #[error(transparent)]
    Unknown(Box<dyn std::error::Error + Send + Sync>),
}

#[derive(Debug, ::thiserror::Error)]
pub enum DeleteByIdMemeError {
    #[error("update-by-id not-found: {id}")]
    MemeNotFound { id: MemeId },
    #[error(transparent)]
    Unknown(Box<dyn std::error::Error + Send + Sync>),
}

#[derive(Debug, ::thiserror::Error)]
pub enum UpdateByIdMemeError {
    #[error("update-by-id not-found: {id}")]
    MemeNotFound { id: MemeId },
    #[error("update-by-id path-taken: {path}")]
    PathTaken { path: MemePath },
    #[error(transparent)]
    Unknown(Box<dyn std::error::Error + Send + Sync>),
}

#[derive(Debug, ::thiserror::Error)]
pub enum FetchMemeByIdError {
    #[error("fetch-by-id not-found: {id}")]
    MemeNotFound { id: MemeId },
    #[error(transparent)]
    Unknown(Box<dyn std::error::Error + Send + Sync>),
}

#[derive(Debug, ::thiserror::Error)]
pub enum InsertUserTagError {
    #[error("insert-user-tag already-exists: {id}")]
    TagIdExists { id: TagId },
    #[error("insert-user-tag name-taken: {name}")]
    NameTaken { name: TagName },
    #[error(transparent)]
    Unknown(Box<dyn std::error::Error + Send + Sync>),
}

#[derive(Debug, ::thiserror::Error)]
pub enum InsertUserTagLinkError {
    #[error("insert-user-tag-link duplicate: tag {tag_id} meme {meme_id}")]
    DuplicateLink { tag_id: TagId, meme_id: MemeId },
    #[error("insert-user-tag-link meme not found: {meme_id}")]
    MemeNotFound { meme_id: MemeId },
    #[error("insert-user-tag-link tag not found: {tag_id}")]
    TagNotFound { tag_id: TagId },
    #[error("insert-user-tag-link both missing: meme {meme_id} tag {tag_id}")]
    BothMissing { meme_id: MemeId, tag_id: TagId },
    #[error(transparent)]
    Unknown(Box<dyn std::error::Error + Send + Sync>),
}

#[derive(Debug, ::thiserror::Error)]
pub enum UpdateUserTagByIdError {
    #[error("update-user-tag-by-id not-found: {id}")]
    TagNotFound { id: TagId },
    #[error("update-user-tag-by-id name-taken: {name}")]
    NameTaken { name: TagName },
    #[error(transparent)]
    Unknown(Box<dyn std::error::Error + Send + Sync>),
}

#[derive(Debug, ::thiserror::Error)]
pub enum DeleteUserTagByIdError {
    #[error("delete-user-tag-by-id not-found: {id}")]
    TagNotFound { id: TagId },
    #[error(transparent)]
    Unknown(Box<dyn std::error::Error + Send + Sync>),
}

#[derive(Debug, ::thiserror::Error)]
pub enum DeleteUserTagLinkError {
    #[error("delete-user-tag-link not-found: tag {tag_id} meme {meme_id}")]
    LinkNotFound { tag_id: TagId, meme_id: MemeId },
    #[error(transparent)]
    Unknown(Box<dyn std::error::Error + Send + Sync>),
}

#[derive(Debug, ::thiserror::Error)]
pub enum UserTagsError {
    #[error(transparent)]
    Unknown(Box<dyn std::error::Error + Send + Sync>),
}

#[cfg(any(test, feature = "testing"))]
pub async fn test_meme_repo<T: MemeRepo>(repo: &T) -> Result<(), MemeRepoError> {
    use ::fake::{Fake, Faker};

    let meme = Faker.fake::<Meme>();

    // Happy path insert, fetch, delete, not-found
    repo.insert_meme(&meme).await?;
    assert_eq!(repo.fetch_meme_by_id(&meme.id).await?, meme);
    repo.delete_meme(&meme.id).await?;
    let Err(missing) = repo.fetch_meme_by_id(&meme.id).await else {
        panic!("expected fetch-by-id to fail after delete");
    };
    assert!(matches!(
        missing,
        FetchMemeByIdError::MemeNotFound { id } if id == meme.id
    ));

    // Attempt to insert with duplicate path
    let mut another = Faker.fake::<Meme>();
    another.path = meme.path.clone();
    repo.insert_meme(&meme).await?;
    let Err(error) = repo.insert_meme(&another).await else {
        panic!("expected insert to fail when path is taken");
    };
    assert!(matches!(
        error,
        InsertMemeError::PathTaken { path } if path == another.path
    ));
    repo.delete_meme(&meme.id).await?;

    // Happy path update-by-id, fetch, delete
    let mut updated = Faker.fake::<Meme>();
    updated.id = meme.id;
    repo.insert_meme(&meme).await?;
    repo.update_meme_by_id(&updated).await?;
    assert_eq!(repo.fetch_meme_by_id(&meme.id).await?, updated);
    repo.delete_meme(&meme.id).await?;

    // Attempt to update with duplicate path
    let mut another = Faker.fake::<Meme>();
    repo.insert_meme(&meme).await?;
    repo.insert_meme(&another).await?;
    another.path = meme.path.clone();
    let Err(error) = repo.update_meme_by_id(&another).await else {
        panic!("expected update-by-id to fail when path is taken");
    };
    assert!(matches!(
        error,
        UpdateByIdMemeError::PathTaken { path } if path == another.path
    ));
    repo.delete_meme(&meme.id).await?;
    repo.delete_meme(&another.id).await?;

    // Creating a tag
    let tag = Faker.fake::<UserTag>();
    repo.insert_user_tag(&tag).await?;

    // Should get id error if we try again
    let tag_with_same_id = UserTag {
        id: tag.id,
        ..Faker.fake()
    };
    let Err(error) = repo.insert_user_tag(&tag_with_same_id).await else {
        panic!("expected insert-user-tag to fail when id already exists");
    };
    dbg!(&error);
    assert!(matches!(
        error,
        InsertUserTagError::TagIdExists { id } if id == tag.id
    ));

    // Same owner cannot have identical tag names
    let another_tag = UserTag {
        name: tag.name.clone(),
        owner_id: tag.owner_id,
        ..Faker.fake()
    };
    let Err(error) = repo.insert_user_tag(&another_tag).await else {
        panic!("expected insert-user-tag to fail when name is taken");
    };
    assert!(matches!(
        error,
        InsertUserTagError::NameTaken { name } if name == another_tag.name
    ));

    // A new owner can have the same tag name though
    let viral_tag = UserTag {
        name: tag.name.clone(),
        ..Faker.fake()
    };
    repo.insert_user_tag(&viral_tag).await?;

    // Should be able to link the tag to a meme
    let link = UserTagLink {
        tag_id: tag.id,
        meme_id: meme.id,
    };
    repo.insert_meme(&meme).await?;
    repo.insert_user_tag_link(&link).await?;

    // Attempting to link the same tag and meme again should fail
    let Err(error) = repo.insert_user_tag_link(&link).await else {
        panic!("expected insert-user-tag-link to fail when duplicate");
    };
    assert!(matches!(
        error,
        InsertUserTagLinkError::DuplicateLink { tag_id, meme_id }
        if tag_id == link.tag_id && meme_id == link.meme_id
    ));

    // expect link with non-existent meme and tag fail with both missing
    let random_link = Faker.fake::<UserTagLink>();
    let Err(error) = repo.insert_user_tag_link(&random_link).await else {
        panic!("expect link with non-existent meme and tag fail with both missing");
    };
    assert!(matches!(
        error,
        InsertUserTagLinkError::BothMissing { tag_id, meme_id }
        if tag_id == random_link.tag_id && meme_id == random_link.meme_id
    ));

    // expect link with non-existent meme to fail with meme not found
    let no_meme_link = UserTagLink {
        meme_id: Faker.fake(),
        ..link
    };
    let Err(error) = repo.insert_user_tag_link(&no_meme_link).await else {
        panic!("expect link with non-existent meme to fail with meme not found");
    };
    assert!(matches!(
        error,
        InsertUserTagLinkError::MemeNotFound { meme_id }
        if meme_id == no_meme_link.meme_id
    ));

    // expect link with non-existent tag to fail with tag not found
    let no_tag_link = UserTagLink {
        tag_id: Faker.fake(),
        ..link
    };
    let Err(error) = repo.insert_user_tag_link(&no_tag_link).await else {
        panic!("expect link with non-existent tag to fail with tag not found");
    };
    assert!(matches!(
        error,
        InsertUserTagLinkError::TagNotFound { tag_id }
        if tag_id == no_tag_link.tag_id
    ));

    Ok(())
}
