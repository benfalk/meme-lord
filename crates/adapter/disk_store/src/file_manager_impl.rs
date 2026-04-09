use crate::DiskStore;
use ::meme::port::file_manager::*;
use ::meme::types::{MemePath, RawFile};

impl FileManager for DiskStore {
    async fn upload(
        &self,
        path: &MemePath,
        file: &RawFile,
    ) -> Result<(), UploadError> {
        let full_path = self.root.join(path.as_str());

        if ::tokio::fs::try_exists(&full_path)
            .await
            .map_err(|e| UploadError::Unknown(e.into()))?
        {
            return Err(UploadError::FileExists { path: path.clone() });
        }

        if let Some(parent) = full_path.parent()
            && parent != self.root
        {
            ::tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| UploadError::Unknown(e.into()))?;
        }

        ::tokio::fs::write(full_path, file)
            .await
            .map_err(|e| UploadError::Unknown(e.into()))
    }

    async fn upsert(
        &self,
        path: &MemePath,
        file: &RawFile,
    ) -> Result<UpsertStatus, UpsertError> {
        let full_path = self.root.join(path.as_str());

        if let Some(parent) = full_path.parent()
            && parent != self.root
        {
            ::tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| UpsertError::Unknown(e.into()))?;
        }

        let existed = ::tokio::fs::try_exists(&full_path)
            .await
            .map_err(|e| UpsertError::Unknown(e.into()))?;

        ::tokio::fs::write(full_path, file)
            .await
            .map_err(|e| UpsertError::Unknown(e.into()))?;

        Ok(if existed {
            UpsertStatus::Updated
        } else {
            UpsertStatus::Added
        })
    }

    async fn delete(&self, path: &MemePath) -> Result<DeleteStatus, DeleteError> {
        let full_path = self.root.join(path.as_str());
        match ::tokio::fs::remove_file(&full_path).await {
            Ok(_) => Ok(DeleteStatus::Deleted),
            Err(e) if e.kind() == ::std::io::ErrorKind::NotFound => {
                Ok(DeleteStatus::NotFound)
            }
            Err(e) => Err(DeleteError::Unknown(e.into())),
        }
    }

    async fn download(&self, path: &MemePath) -> Result<RawFile, DownloadError> {
        let full_path = self.root.join(path.as_str());
        let bytes =
            ::tokio::fs::read(full_path)
                .await
                .map_err(|e| match e.kind() {
                    ::std::io::ErrorKind::NotFound => {
                        DownloadError::FileNotFound { path: path.clone() }
                    }
                    _ => DownloadError::Unknown(e.into()),
                })?;
        Ok(RawFile::from(bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::tempfile::TempDir;

    #[rstest::fixture]
    fn empty_dir() -> TempDir {
        TempDir::new().unwrap()
    }

    #[rstest::fixture]
    fn with_test_file(empty_dir: TempDir) -> TempDir {
        let path = MemePath::from("test.txt");
        let file = RawFile::from("Hello, world!");
        ::std::fs::write(empty_dir.path().join(path.as_str()), &file).unwrap();
        empty_dir
    }

    #[tokio::test]
    #[rstest::rstest]
    async fn upload_new_file_happy_path(empty_dir: TempDir) {
        let store = DiskStore::new(empty_dir.path());
        let path = MemePath::from("test.txt");
        let file = RawFile::from("Hello, world!");

        store.upload(&path, &file).await.unwrap();

        let full_path = store.root.join(path.as_str());
        assert!(full_path.exists());
        let content = ::tokio::fs::read(full_path).await.unwrap();
        assert_eq!(content, file);
    }

    #[tokio::test]
    #[rstest::rstest]
    async fn upload_new_file_with_nested_dirs(empty_dir: TempDir) {
        let store = DiskStore::new(empty_dir.path());
        let path = MemePath::from("a/b/c/test.txt");
        let file = RawFile::from("Hello, world!");

        store.upload(&path, &file).await.unwrap();

        let full_path = store.root.join(path.as_str());
        assert!(full_path.exists());
        let content = ::tokio::fs::read(full_path).await.unwrap();
        assert_eq!(content, file);
    }

    #[tokio::test]
    #[rstest::rstest]
    async fn upload_existing_file(with_test_file: TempDir) {
        let store = DiskStore::new(with_test_file.path());
        let path = MemePath::from("test.txt");
        let file = RawFile::from("Hello, world!");

        let err = store.upload(&path, &file).await.unwrap_err();
        assert!(matches!(err, UploadError::FileExists { path: p } if p == path));
    }

    #[tokio::test]
    #[rstest::rstest]
    async fn upsert_new_file_happy_path(empty_dir: TempDir) {
        let store = DiskStore::new(empty_dir.path());
        let path = MemePath::from("test.txt");
        let file = RawFile::from("Hello, world!");

        let status = store.upsert(&path, &file).await.unwrap();
        assert_eq!(status, UpsertStatus::Added);

        let full_path = store.root.join(path.as_str());
        assert!(full_path.exists());
        let content = ::tokio::fs::read(full_path).await.unwrap();
        assert_eq!(content, file);
    }

    #[tokio::test]
    #[rstest::rstest]
    async fn upsert_existing_file(with_test_file: TempDir) {
        let store = DiskStore::new(with_test_file.path());
        let path = MemePath::from("test.txt");
        let file = RawFile::from("Hello, world! (Again)");

        let status = store.upsert(&path, &file).await.unwrap();
        assert_eq!(status, UpsertStatus::Updated);

        let full_path = store.root.join(path.as_str());
        assert!(full_path.exists());
        let content = ::tokio::fs::read(full_path).await.unwrap();
        assert_eq!(content, file);
    }

    #[tokio::test]
    #[rstest::rstest]
    async fn delete_existing_file(with_test_file: TempDir) {
        let store = DiskStore::new(with_test_file.path());
        let path = MemePath::from("test.txt");

        let status = store.delete(&path).await.unwrap();
        assert_eq!(status, DeleteStatus::Deleted);

        let full_path = store.root.join(path.as_str());
        assert!(!full_path.exists());
    }

    #[tokio::test]
    #[rstest::rstest]
    async fn delete_nonexistent_file(empty_dir: TempDir) {
        let store = DiskStore::new(empty_dir.path());
        let path = MemePath::from("nonexistent.txt");

        let status = store.delete(&path).await.unwrap();
        assert_eq!(status, DeleteStatus::NotFound);
    }

    #[tokio::test]
    #[rstest::rstest]
    async fn download_existing_file(with_test_file: TempDir) {
        let store = DiskStore::new(with_test_file.path());
        let path = MemePath::from("test.txt");

        let file = store.download(&path).await.unwrap();
        assert_eq!(file, RawFile::from("Hello, world!"));
    }

    #[tokio::test]
    #[rstest::rstest]
    async fn download_nonexistent_file(empty_dir: TempDir) {
        let store = DiskStore::new(empty_dir.path());
        let path = MemePath::from("nonexistent.txt");

        let err = store.download(&path).await.unwrap_err();
        assert!(matches!(
            err,
            DownloadError::FileNotFound { path: p } if p == path
        ));
    }
}
