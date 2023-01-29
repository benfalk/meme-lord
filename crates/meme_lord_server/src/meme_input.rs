use actix_multipart::{Field, Multipart};
use futures_util::TryStreamExt;
use serde::Deserialize;
use std::{fmt::Debug, io::Write};
use meme_lord_core::{Meme, MemeDetails};

type AnyError = Box<dyn std::error::Error>;

#[derive(Default)]
pub struct MemeInput {
    id: Option<String>,
    data: Option<Vec<u8>>,
    details: MemeDetailsInput,
}

impl Debug for MemeInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MemeInput")
            .field("id", &self.id)
            .field(
                "data",
                &self.data.as_ref().map(|d| d.len()).unwrap_or_default(),
            )
            .field("details", &self.details)
            .finish()
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct MemeDetailsInput {
    meta: Option<String>,
    caption: Option<String>,
}

impl MemeInput {
    pub async fn try_from(mut data: Multipart) -> Result<Self, AnyError> {
        let mut input = Self::default();

        while let Some(field) = data.try_next().await? {
            match field.name() {
                "file" => {
                    let (data, id) = get_file_and_id(field).await?;
                    input.id = Some(id);
                    input.data = Some(data);
                }
                "details" => {
                    input.details = get_details(field).await?;
                }
                _ => (),
            }
        }

        Ok(input)
    }

    pub fn is_valid(&self) -> bool {
        self.id.is_some()
    }

    pub fn to_meme(self) -> Meme {
        Meme::new(self.id.unwrap(), self.data.unwrap())
    }
}

async fn get_file_and_id(mut field: Field) -> Result<(Vec<u8>, String), AnyError> {
    let mut data = vec![];
    let id = field
        .content_disposition()
        .get_filename()
        .unwrap_or_default()
        .to_owned();
    while let Some(chunk) = field.try_next().await? {
        data.write_all(&chunk)?;
    }
    Ok((data, id))
}

async fn get_details(mut field: Field) -> Result<MemeDetailsInput, AnyError> {
    let mut data = vec![];
    while let Some(chunk) = field.try_next().await? {
        data.write_all(&chunk)?;
    }
    Ok(serde_json::from_slice(&data)?)
}
