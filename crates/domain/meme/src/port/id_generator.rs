use crate::types::{MemeId, TagId};

#[cfg_attr(any(test, feature = "testing"), ::mockall::automock)]
pub trait IdGenerator: Send + Sync {
    fn generate_meme_id(
        &self,
    ) -> impl Future<Output = Result<MemeId, GenerateMemeIdError>> + Send;

    fn generate_tag_id(
        &self,
    ) -> impl Future<Output = Result<TagId, GenerateTagIdError>> + Send;
}

#[derive(Debug, ::thiserror::Error)]
#[error("generate-meme-id: {source}")]
pub struct GenerateMemeIdError {
    pub source: Box<dyn std::error::Error + Send + Sync>,
}

#[derive(Debug, ::thiserror::Error)]
#[error("generate-tag-id: {source}")]
pub struct GenerateTagIdError {
    pub source: Box<dyn std::error::Error + Send + Sync>,
}

#[cfg(any(test, feature = "testing"))]
pub async fn test_id_generator<G: IdGenerator>(
    generator: &G,
) -> Result<(), GenerateMemeIdError> {
    for pass in 1..=100_000 {
        let meme_one = generator
            .generate_meme_id()
            .await
            .expect("failed to generate meme id");

        let meme_two = generator
            .generate_meme_id()
            .await
            .expect("failed to generate meme id");

        assert_ne!(meme_one, meme_two, "ids are not the same on pass {pass}");
    }
    Ok(())
}
