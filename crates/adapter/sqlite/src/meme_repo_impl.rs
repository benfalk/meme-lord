use crate::Sqlite;
use ::identity::UserId;
use ::meme::entity::{Meme, UserTag, UserTagLink};
use ::meme::port::meme_repo::*;
use ::meme::types::{ByteSize, MemeId, TagId};
use ::sqlx::SqlitePool;
use ::uuid::Uuid;

type MemeRow = (Vec<u8>, Vec<u8>, String, Option<String>, i64);
type UserTagRow = (Vec<u8>, Vec<u8>, String);

impl MemeRepo for Sqlite {
    async fn fetch_meme_by_id(
        &self,
        id: &MemeId,
    ) -> Result<Meme, FetchMemeByIdError> {
        let row: Option<MemeRow> = sqlx::query_as(
            r#"
            SELECT id, owner_id, meme_path, caption, file_size
            FROM memes
            WHERE id = ?
            "#,
        )
        .bind(id.into_inner().as_bytes().as_slice())
        .fetch_optional(&self.pool)
        .await
        .map_err(|err| FetchMemeByIdError::Unknown(Box::new(err)))?;

        let Some((id, owner_id, meme_path, caption, file_size)) = row else {
            return Err(FetchMemeByIdError::MemeNotFound { id: *id });
        };

        let id = Uuid::from_slice(&id).unwrap();
        let owner_id = Uuid::from_slice(&owner_id).unwrap();

        Ok(Meme {
            id: MemeId::try_from(id).unwrap(),
            owner_id: UserId::try_from(owner_id).unwrap(),
            path: meme_path.into(),
            file_size: ByteSize::b(file_size as u64),
            caption: caption.map(Into::into),
        })
    }

    async fn insert_meme(&self, meme: &Meme) -> Result<(), InsertMemeError> {
        sqlx::query(
            r#"
            INSERT INTO memes (id, owner_id, meme_path, caption, file_size)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(meme.id.into_inner().as_bytes().as_slice())
        .bind(meme.owner_id.into_inner().as_bytes().as_slice())
        .bind(meme.path.as_str())
        .bind(meme.caption.as_ref().map(|c| c.as_str()))
        .bind(meme.file_size.as_u64() as i64)
        .execute(&self.pool)
        .await
        .map_err(|err| {
            if let Some(db_err) = err.as_database_error() {
                if db_err.code().as_deref() == Some("1555") {
                    return InsertMemeError::MemeAlreadyExists { id: meme.id };
                }

                if db_err.code().as_deref() == Some("2067") {
                    return InsertMemeError::PathTaken {
                        path: meme.path.clone(),
                    };
                }
            }

            InsertMemeError::Unknown(Box::new(err))
        })?;

        Ok(())
    }

    async fn delete_meme(&self, id: &MemeId) -> Result<(), DeleteByIdMemeError> {
        let results = ::sqlx::query("DELETE FROM memes WHERE id = ?")
            .bind(id.into_inner().as_bytes().as_slice())
            .execute(&self.pool)
            .await
            .map_err(|err| DeleteByIdMemeError::Unknown(Box::new(err)))?;

        if results.rows_affected() == 0 {
            return Err(DeleteByIdMemeError::MemeNotFound { id: *id });
        }

        Ok(())
    }

    async fn update_meme_by_id(
        &self,
        meme: &Meme,
    ) -> Result<(), UpdateByIdMemeError> {
        let results = ::sqlx::query(
            r#"
            UPDATE memes
            SET owner_id = ?, meme_path = ?, caption = ?, file_size = ?
            WHERE id = ?
            "#,
        )
        .bind(meme.owner_id.into_inner().as_bytes().as_slice())
        .bind(meme.path.as_str())
        .bind(meme.caption.as_ref().map(|c| c.as_str()))
        .bind(meme.file_size.as_u64() as i64)
        .bind(meme.id.into_inner().as_bytes().as_slice())
        .execute(&self.pool)
        .await
        .map_err(|err| {
            if let Some(db_err) = err.as_database_error()
                && db_err.code().as_deref() == Some("2067")
            {
                return UpdateByIdMemeError::PathTaken {
                    path: meme.path.clone(),
                };
            }

            UpdateByIdMemeError::Unknown(Box::new(err))
        })?;

        if results.rows_affected() == 0 {
            return Err(UpdateByIdMemeError::MemeNotFound { id: meme.id });
        }

        Ok(())
    }

    async fn insert_user_tag(
        &self,
        tag: &UserTag,
    ) -> Result<(), InsertUserTagError> {
        sqlx::query(
            r#"
            INSERT INTO user_tags (id, owner_id, name)
            VALUES (?, ?, ?)
            "#,
        )
        .bind(tag.id.into_inner().as_bytes().as_slice())
        .bind(tag.owner_id.into_inner().as_bytes().as_slice())
        .bind(tag.name.as_str())
        .execute(&self.pool)
        .await
        .map_err(|err| {
            if let Some(db_err) = err.as_database_error() {
                if db_err.code().as_deref() == Some("1555") {
                    return InsertUserTagError::TagIdExists { id: tag.id };
                }

                if db_err.code().as_deref() == Some("2067") {
                    return InsertUserTagError::NameTaken {
                        name: tag.name.clone(),
                    };
                }
            }

            InsertUserTagError::Unknown(Box::new(err))
        })?;

        Ok(())
    }

    async fn insert_user_tag_link(
        &self,
        tag_link: &UserTagLink,
    ) -> Result<(), InsertUserTagLinkError> {
        let result = sqlx::query(
            r#"
            INSERT INTO user_tag_links (meme_id, tag_id)
            VALUES (?, ?)
            "#,
        )
        .bind(tag_link.meme_id.into_inner().as_bytes().as_slice())
        .bind(tag_link.tag_id.into_inner().as_bytes().as_slice())
        .execute(&self.pool)
        .await;

        let Err(err) = result else { return Ok(()) };
        let db_err = err.as_database_error();
        let code = db_err.and_then(|e| e.code());
        let insert_err = match code.as_deref() {
            Some("1555") => InsertUserTagLinkError::DuplicateLink {
                tag_id: tag_link.tag_id,
                meme_id: tag_link.meme_id,
            },
            Some("787") => {
                // sqlite does not provide a way to determine which FK
                // failed, so we have to check manually. This is not ideal,
                // but it's the best we can do.
                let checks = ::tokio::try_join!(
                    tag_exists(&self.pool, &tag_link.tag_id),
                    meme_exists(&self.pool, &tag_link.meme_id),
                )
                .unwrap();
                match checks {
                    (true, false) => InsertUserTagLinkError::MemeNotFound {
                        meme_id: tag_link.meme_id,
                    },
                    (false, true) => InsertUserTagLinkError::TagNotFound {
                        tag_id: tag_link.tag_id,
                    },
                    (false, false) => InsertUserTagLinkError::BothMissing {
                        meme_id: tag_link.meme_id,
                        tag_id: tag_link.tag_id,
                    },
                    _ => unreachable!("neither tag or meme is missing..."),
                }
            }
            _ => InsertUserTagLinkError::Unknown(Box::new(err)),
        };
        Err(insert_err)
    }

    async fn delete_user_tag_link(
        &self,
        tag_link: &UserTagLink,
    ) -> Result<(), DeleteUserTagLinkError> {
        let results = sqlx::query(
            r#"
            DELETE FROM user_tag_links
            WHERE meme_id = ? AND tag_id = ?
            "#,
        )
        .bind(tag_link.meme_id.into_inner().as_bytes().as_slice())
        .bind(tag_link.tag_id.into_inner().as_bytes().as_slice())
        .execute(&self.pool)
        .await
        .map_err(|err| DeleteUserTagLinkError::Unknown(Box::new(err)))?;

        if results.rows_affected() == 0 {
            return Err(DeleteUserTagLinkError::LinkNotFound {
                meme_id: tag_link.meme_id,
                tag_id: tag_link.tag_id,
            });
        }

        Ok(())
    }

    async fn update_user_tag_by_id(
        &self,
        tag: &UserTag,
    ) -> Result<(), UpdateUserTagByIdError> {
        let results = sqlx::query(
            r#"
            UPDATE user_tags
            SET owner_id = ?, name = ?
            WHERE id = ?
            "#,
        )
        .bind(tag.owner_id.into_inner().as_bytes().as_slice())
        .bind(tag.name.as_str())
        .bind(tag.id.into_inner().as_bytes().as_slice())
        .execute(&self.pool)
        .await
        .map_err(|err| {
            if let Some(db_err) = err.as_database_error()
                && db_err.code().as_deref() == Some("2067")
            {
                return UpdateUserTagByIdError::NameTaken {
                    name: tag.name.clone(),
                };
            }

            UpdateUserTagByIdError::Unknown(Box::new(err))
        })?;

        if results.rows_affected() == 0 {
            return Err(UpdateUserTagByIdError::TagNotFound { id: tag.id });
        }

        Ok(())
    }

    async fn delete_user_tag_by_id(
        &self,
        id: &TagId,
    ) -> Result<(), DeleteUserTagByIdError> {
        let results = sqlx::query("DELETE FROM user_tags WHERE id = ?")
            .bind(id.into_inner().as_bytes().as_slice())
            .execute(&self.pool)
            .await
            .map_err(|err| DeleteUserTagByIdError::Unknown(Box::new(err)))?;

        if results.rows_affected() == 0 {
            return Err(DeleteUserTagByIdError::TagNotFound { id: *id });
        }

        Ok(())
    }

    async fn user_tags(
        &self,
        owner_id: &UserId,
    ) -> Result<Vec<UserTag>, UserTagsError> {
        let rows: Vec<UserTagRow> = sqlx::query_as(
            r#"
            SELECT id, owner_id, name
            FROM user_tags
            WHERE owner_id = ?
            "#,
        )
        .bind(owner_id.into_inner().as_bytes().as_slice())
        .fetch_all(&self.pool)
        .await
        .map_err(|err| UserTagsError::Unknown(Box::new(err)))?;

        Ok(rows
            .into_iter()
            .map(|(id, owner_id, name)| {
                let id = Uuid::from_slice(&id).unwrap();
                let owner_id = Uuid::from_slice(&owner_id).unwrap();
                UserTag {
                    id: TagId::try_from(id).unwrap(),
                    owner_id: UserId::try_from(owner_id).unwrap(),
                    name: name.into(),
                }
            })
            .collect())
    }
}

async fn tag_exists(pool: &SqlitePool, id: &TagId) -> Result<bool, sqlx::Error> {
    let row: Option<(i64,)> = sqlx::query_as(
        r#"
        SELECT 1
        FROM user_tags
        WHERE id = ?
        "#,
    )
    .bind(id.into_inner().as_bytes().as_slice())
    .fetch_optional(pool)
    .await?;

    Ok(row.is_some())
}

async fn meme_exists(pool: &SqlitePool, id: &MemeId) -> Result<bool, sqlx::Error> {
    let row: Option<(i64,)> = sqlx::query_as(
        r#"
        SELECT 1
        FROM memes
        WHERE id = ?
        "#,
    )
    .bind(id.into_inner().as_bytes().as_slice())
    .fetch_optional(pool)
    .await?;

    Ok(row.is_some())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::fake::{Fake, Faker};
    use ::sqlx::SqlitePool;

    fn single_meme_id() -> MemeId {
        "01890f4c5b7a7cc2bb7b8e8b8e7e9c3a"
            .parse::<MemeId>()
            .unwrap()
    }

    fn single_meme_owner_id() -> UserId {
        "01890f4d2e3b7e3a8c1d9b2a4f6e1d2c"
            .parse::<UserId>()
            .unwrap()
    }

    fn single_tag_id() -> TagId {
        "01890f4e5b7a7cc2bb7b8e8b8e7e9c3e".parse::<TagId>().unwrap()
    }

    fn single_tag_owner_id() -> UserId {
        single_meme_owner_id()
    }

    #[sqlx::test(fixtures("../fixtures/single-meme.sql"))]
    async fn fetch_meme_by_id_success(pool: SqlitePool) {
        let meme_id = single_meme_id();
        let sqlite = Sqlite::new(pool);

        let meme = sqlite.fetch_meme_by_id(&meme_id).await.unwrap();

        assert_eq!(meme.id, meme_id);
        assert_eq!(meme.owner_id, single_meme_owner_id());
        assert_eq!(meme.path, "/something-of-a-meme-lord-myself.jpg");
        assert_eq!(meme.file_size, ByteSize::b(127_000));
        assert_eq!(
            meme.caption.as_ref().map(|c| c.as_str()),
            Some("I'm something of a meme lord myself")
        );
    }

    #[sqlx::test]
    async fn fetch_meme_by_id_not_found(pool: SqlitePool) {
        let meme_id = MemeId::generate();
        let sqlite = Sqlite::new(pool);
        let err = sqlite.fetch_meme_by_id(&meme_id).await.unwrap_err();
        assert!(matches!(
            err,
            FetchMemeByIdError::MemeNotFound { id } if id == meme_id
        ));
    }

    #[sqlx::test]
    async fn insert_meme_success(pool: SqlitePool) {
        let sqlite = Sqlite::new(pool);
        let meme = Faker.fake::<Meme>();
        sqlite.insert_meme(&meme).await.unwrap();
    }

    #[sqlx::test(fixtures("../fixtures/single-meme.sql"))]
    async fn insert_meme_duplicate_id(pool: SqlitePool) {
        let sqlite = Sqlite::new(pool);
        let meme = Meme {
            id: single_meme_id(),
            ..Faker.fake::<Meme>()
        };
        let err = sqlite.insert_meme(&meme).await.unwrap_err();
        assert!(matches!(
            err,
            InsertMemeError::MemeAlreadyExists { id } if id == single_meme_id()
        ));
    }

    #[sqlx::test(fixtures("../fixtures/single-meme.sql"))]
    async fn insert_meme_duplicate_path(pool: SqlitePool) {
        let sqlite = Sqlite::new(pool);
        let meme = Meme {
            path: "/something-of-a-meme-lord-myself.jpg".into(),
            ..Faker.fake::<Meme>()
        };
        let err = sqlite.insert_meme(&meme).await.unwrap_err();
        assert!(matches!(
            err,
            InsertMemeError::PathTaken { path } if path == "/something-of-a-meme-lord-myself.jpg"
        ));
    }

    #[sqlx::test(fixtures("../fixtures/single-meme.sql"))]
    async fn delete_meme_success(pool: SqlitePool) {
        let sqlite = Sqlite::new(pool);
        let meme_id = single_meme_id();
        sqlite.delete_meme(&meme_id).await.unwrap();
    }

    #[sqlx::test]
    async fn delete_meme_not_found(pool: SqlitePool) {
        let sqlite = Sqlite::new(pool);
        let meme_id = MemeId::generate();
        let err = sqlite.delete_meme(&meme_id).await.unwrap_err();
        assert!(matches!(
            err,
            DeleteByIdMemeError::MemeNotFound { id } if id == meme_id
        ));
    }

    #[sqlx::test(fixtures("../fixtures/single-meme.sql"))]
    async fn update_meme_by_id_success(pool: SqlitePool) {
        let sqlite = Sqlite::new(pool);
        let updated = Meme {
            id: single_meme_id(),
            ..Faker.fake::<Meme>()
        };
        sqlite.update_meme_by_id(&updated).await.unwrap();
    }

    #[sqlx::test(fixtures("../fixtures/single-meme.sql"))]
    async fn update_meme_by_id_not_found(pool: SqlitePool) {
        let sqlite = Sqlite::new(pool);
        let meme = Faker.fake::<Meme>();
        let err = sqlite.update_meme_by_id(&meme).await.unwrap_err();
        assert!(matches!(
            err,
            UpdateByIdMemeError::MemeNotFound { id } if id == meme.id
        ));
    }

    #[sqlx::test(fixtures("../fixtures/multi-memes.sql"))]
    async fn update_meme_duplicate_path(pool: SqlitePool) {
        let sqlite = Sqlite::new(pool);
        let meme = Meme {
            id: single_meme_id(),
            path: "/another-swing.jpg".into(),
            ..Faker.fake::<Meme>()
        };
        let err = sqlite.update_meme_by_id(&meme).await.unwrap_err();
        dbg!(&err);
        assert!(matches!(
            err,
            UpdateByIdMemeError::PathTaken { path } if path == "/another-swing.jpg"
        ));
    }

    #[sqlx::test]
    async fn insert_user_tag_success(pool: SqlitePool) {
        let sqlite = Sqlite::new(pool);
        let tag = Faker.fake::<UserTag>();
        sqlite.insert_user_tag(&tag).await.unwrap();
    }

    #[sqlx::test(fixtures("../fixtures/single-tag.sql"))]
    async fn insert_user_tag_id_exists(pool: SqlitePool) {
        let sqlite = Sqlite::new(pool);
        let tag = UserTag {
            id: single_tag_id(),
            ..Faker.fake::<UserTag>()
        };
        let err = sqlite.insert_user_tag(&tag).await.unwrap_err();
        assert!(matches!(
            err,
            InsertUserTagError::TagIdExists { id } if id == single_tag_id()
        ));
    }

    #[sqlx::test(fixtures("../fixtures/single-tag.sql"))]
    async fn insert_user_tag_name_taken(pool: SqlitePool) {
        let sqlite = Sqlite::new(pool);
        let tag = UserTag {
            owner_id: single_tag_owner_id(),
            name: "nsfw".into(),
            ..Faker.fake::<UserTag>()
        };
        let err = sqlite.insert_user_tag(&tag).await.unwrap_err();
        assert!(matches!(
            err,
            InsertUserTagError::NameTaken { name } if name == "nsfw"
        ));
    }

    #[sqlx::test(fixtures("../fixtures/single-meme-and-tag.sql"))]
    async fn insert_user_tag_link_success(pool: SqlitePool) {
        let sqlite = Sqlite::new(pool);
        let link = UserTagLink {
            meme_id: single_meme_id(),
            tag_id: single_tag_id(),
        };
        sqlite.insert_user_tag_link(&link).await.unwrap();
    }

    #[sqlx::test(fixtures("../fixtures/single-meme-and-tag-linked.sql"))]
    async fn insert_user_tag_link_duplicate(pool: SqlitePool) {
        let sqlite = Sqlite::new(pool);
        let link = UserTagLink {
            meme_id: single_meme_id(),
            tag_id: single_tag_id(),
        };
        let err = sqlite.insert_user_tag_link(&link).await.unwrap_err();
        dbg!(&err);
        assert!(matches!(
            err,
            InsertUserTagLinkError::DuplicateLink {
                tag_id,
                meme_id
            } if tag_id == single_tag_id() && meme_id == single_meme_id()
        ));
    }

    #[sqlx::test(fixtures("../fixtures/single-tag.sql"))]
    async fn insert_user_tag_link_meme_missing(pool: SqlitePool) {
        let sqlite = Sqlite::new(pool);
        let link = UserTagLink {
            meme_id: single_meme_id(),
            tag_id: single_tag_id(),
        };
        let err = sqlite.insert_user_tag_link(&link).await.unwrap_err();
        dbg!(&err);
        assert!(matches!(
            err,
            InsertUserTagLinkError::MemeNotFound {
                meme_id
            } if meme_id == single_meme_id()
        ));
    }

    #[sqlx::test(fixtures("../fixtures/single-meme.sql"))]
    async fn insert_user_tag_link_tag_missing(pool: SqlitePool) {
        let sqlite = Sqlite::new(pool);
        let link = UserTagLink {
            meme_id: single_meme_id(),
            tag_id: single_tag_id(),
        };
        let err = sqlite.insert_user_tag_link(&link).await.unwrap_err();
        dbg!(&err);
        assert!(matches!(
            err,
            InsertUserTagLinkError::TagNotFound {
                tag_id
            } if tag_id == single_tag_id()
        ));
    }

    #[sqlx::test]
    async fn insert_user_tag_link_both_missing(pool: SqlitePool) {
        let sqlite = Sqlite::new(pool);
        let link = UserTagLink {
            meme_id: single_meme_id(),
            tag_id: single_tag_id(),
        };
        let err = sqlite.insert_user_tag_link(&link).await.unwrap_err();
        dbg!(&err);
        assert!(matches!(
            err,
            InsertUserTagLinkError::BothMissing {
                meme_id,
                tag_id
            } if meme_id == single_meme_id() && tag_id == single_tag_id()
        ));
    }

    #[sqlx::test]
    async fn meme_repo_test_suite(pool: SqlitePool) {
        let sqlite = Sqlite::new(pool);
        test_meme_repo(&sqlite)
            .await
            .expect("meme repo test suite failed");
    }

    #[sqlx::test(fixtures("../fixtures/single-tag.sql"))]
    async fn update_user_tag_by_id_success(pool: SqlitePool) {
        let sqlite = Sqlite::new(pool);
        let updated = UserTag {
            id: single_tag_id(),
            owner_id: single_tag_owner_id(),
            name: "rofl".into(),
        };
        sqlite.update_user_tag_by_id(&updated).await.unwrap();
    }

    #[sqlx::test(fixtures("../fixtures/multi-tags.sql"))]
    async fn update_user_tag_by_id_name_taken(pool: SqlitePool) {
        let sqlite = Sqlite::new(pool);
        let updated = UserTag {
            id: single_tag_id(),
            owner_id: single_tag_owner_id(),
            name: "programming".into(),
        };
        let err = sqlite.update_user_tag_by_id(&updated).await.unwrap_err();
        assert!(matches!(
            err,
            UpdateUserTagByIdError::NameTaken { name } if name == "programming"
        ));
    }

    #[sqlx::test]
    async fn update_user_tag_by_id_not_found(pool: SqlitePool) {
        let sqlite = Sqlite::new(pool);
        let tag = Faker.fake::<UserTag>();
        let err = sqlite.update_user_tag_by_id(&tag).await.unwrap_err();
        assert!(matches!(
            err,
            UpdateUserTagByIdError::TagNotFound { id } if id == tag.id
        ));
    }

    #[sqlx::test(fixtures("../fixtures/single-tag.sql"))]
    async fn delete_user_tag_success(pool: SqlitePool) {
        let sqlite = Sqlite::new(pool);
        let tag_id = single_tag_id();
        sqlite.delete_user_tag_by_id(&tag_id).await.unwrap();
    }

    #[sqlx::test]
    async fn delete_user_tag_not_found(pool: SqlitePool) {
        let sqlite = Sqlite::new(pool);
        let tag_id = TagId::generate();
        let err = sqlite.delete_user_tag_by_id(&tag_id).await.unwrap_err();
        assert!(matches!(
            err,
            DeleteUserTagByIdError::TagNotFound { id } if id == tag_id
        ));
    }

    #[sqlx::test(fixtures("../fixtures/single-meme-and-tag-linked.sql"))]
    async fn delete_user_tag_link_success(pool: SqlitePool) {
        let sqlite = Sqlite::new(pool);
        let link = UserTagLink {
            meme_id: single_meme_id(),
            tag_id: single_tag_id(),
        };
        sqlite.delete_user_tag_link(&link).await.unwrap();
    }

    #[sqlx::test]
    async fn delete_user_tag_link_not_found(pool: SqlitePool) {
        let sqlite = Sqlite::new(pool);
        let link = UserTagLink {
            meme_id: single_meme_id(),
            tag_id: single_tag_id(),
        };
        let err = sqlite.delete_user_tag_link(&link).await.unwrap_err();
        assert!(matches!(
            err,
            DeleteUserTagLinkError::LinkNotFound {
                meme_id,
                tag_id
            } if meme_id == single_meme_id() && tag_id == single_tag_id()
        ));
    }

    #[sqlx::test(fixtures("../fixtures/multi-tags.sql"))]
    async fn user_tags_success_some(pool: SqlitePool) {
        let sqlite = Sqlite::new(pool);
        let owner_id = single_tag_owner_id();
        let tags = sqlite.user_tags(&owner_id).await.unwrap();
        assert_eq!(tags.len(), 2);
        assert!(tags.iter().any(|t| t.name == "nsfw"));
        assert!(tags.iter().any(|t| t.name == "programming"));
    }

    #[sqlx::test]
    async fn user_tags_success_none(pool: SqlitePool) {
        let sqlite = Sqlite::new(pool);
        let owner_id = UserId::generate();
        let tags = sqlite.user_tags(&owner_id).await.unwrap();
        assert!(tags.is_empty());
    }
}
