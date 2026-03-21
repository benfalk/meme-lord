pub mod create_meme;

mod prelude {
    pub(crate) use crate::port::file_uploader::FileManager;
    pub(crate) use crate::port::meme_repo::MemeRepo;
    pub(crate) use crate::sevice::{Command, Env};
}
