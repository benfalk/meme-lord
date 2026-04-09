use ::std::path::PathBuf;

pub struct DiskStore {
    pub(crate) root: PathBuf,
}

impl DiskStore {
    pub fn new<T>(root: T) -> Self
    where
        T: Into<PathBuf>,
    {
        // TODO: check that it's a directory and that it exists
        Self { root: root.into() }
    }
}
