use crate::meme::{Meme, MemeDetails};
use crate::store::Store;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

type ArcLock<T> = Arc<RwLock<T>>;
type DetailMap = HashMap<String, MemeDetails>;

#[derive(Debug, Clone, Default)]
pub struct FileStore {
    directory: ArcLock<Directory>,
    details: ArcLock<DetailMap>,
}

impl FileStore {
    pub fn new<T: ToString>(dir: T) -> Self {
        let directory = Directory::new(dir.to_string());

        let details = directory
            .get_details()
            .into_iter()
            .map(|d| (d.id.clone(), d))
            .collect();

        Self {
            directory: Arc::new(RwLock::new(directory)),
            details: Arc::new(RwLock::new(details)),
        }
    }
}

impl Store for FileStore {
    fn details(&self) -> Vec<MemeDetails> {
        self.details.read().unwrap().values().cloned().collect()
    }

    fn get(&self, id: &str) -> Option<Meme> {
        let details = self.details.read().unwrap().get(id).cloned()?;

        let data = self.directory.read().unwrap().get_meme_data(id);

        Some(Meme::new_with_details(data, details))
    }

    fn put(&self, meme: Meme) {
        let (data, details) = meme.split();
        let mut dir = self.directory.write().unwrap();

        dir.save_meme_data(&details.id, &data);

        self.details
            .write()
            .unwrap()
            .insert(details.id.clone(), details);

        let detail_reader = self.details.read().unwrap();
        let details = detail_reader.values().collect();
        dir.save_details(details);
    }
}

#[derive(Debug, Clone, Default)]
pub struct Directory {
    root_dir: String,
    details: String,
}

impl Directory {
    fn new(mut root_dir: String) -> Self {
        if !root_dir.ends_with('/') {
            root_dir.push('/');
        }

        let details = format!("{root_dir}.meme-detils");

        Self { root_dir, details }
    }

    fn get_details(&self) -> Vec<MemeDetails> {
        let file = match std::fs::File::open(&self.details) {
            Ok(f) => f,
            Err(_) => return vec![],
        };
        serde_json::from_reader(file).unwrap()
    }

    fn save_details(&mut self, details: Vec<&MemeDetails>) {
        let data = serde_json::to_string(&details).unwrap();
        std::fs::write(&self.details, &data).unwrap();
    }

    fn get_meme_data(&self, id: &str) -> Vec<u8> {
        std::fs::read(&[&self.root_dir, id].join("")).unwrap()
    }

    fn save_meme_data(&mut self, id: &str, data: &Vec<u8>) {
        std::fs::write(&[&self.root_dir, id].join(""), data).unwrap();
    }
}
