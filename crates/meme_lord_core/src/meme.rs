use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Clone)]
pub struct Meme {
    data: Vec<u8>,
    details: MemeDetails,
}

impl Debug for Meme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Meme")
            .field("data", &self.data.len())
            .field("details", &self.details)
            .finish()
    }
}

impl Meme {
    pub fn new(id: String, data: Vec<u8>) -> Self {
        Self {
            data,
            details: MemeDetails {
                id,
                ..MemeDetails::default()
            },
        }
    }

    pub fn new_with_details(data: Vec<u8>, details: MemeDetails) -> Self {
        Self {
            data,
            details,
        }
    }

    pub fn into_data(self) -> Vec<u8> {
        self.data
    }

    pub fn id(&self) -> &String {
        &self.details.id
    }

    pub fn caption(&self) -> &String {
        &self.details.caption
    }

    pub fn meta(&self) -> &String {
        &self.details.meta
    }

    pub fn details(&self) -> &MemeDetails {
        &self.details
    }

    pub fn data(&self) -> &Vec<u8> {
        &self.data
    }

    pub fn split(self) -> (Vec<u8>, MemeDetails) {
        (self.data, self.details)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemeDetails {
    pub id: String,
    pub caption: String,
    pub meta: String,
}
