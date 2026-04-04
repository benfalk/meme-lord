use crate::port::id_generator::*;
use crate::types::{MemeId, TagId};

#[derive(Debug, Default)]
#[non_exhaustive]
pub struct StdIdGenerator;

impl IdGenerator for StdIdGenerator {
    async fn generate_meme_id(&self) -> Result<MemeId, GenerateMemeIdError> {
        Ok(MemeId::generate())
    }

    async fn generate_tag_id(&self) -> Result<TagId, GenerateTagIdError> {
        Ok(TagId::generate())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn works_as_an_id_generator() {
        let generator = StdIdGenerator::default();
        test_id_generator(&generator)
            .await
            .expect("it to pass id generatio mustard");
    }
}
