use crate::meme::{Meme, MemeDetails};

pub trait Store {
    fn get(&self, id: &str) -> Option<Meme>;
    fn put(&self, meme: Meme);
    fn details(&self) -> Vec<MemeDetails>;
}
