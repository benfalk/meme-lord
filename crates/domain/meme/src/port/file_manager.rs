use crate::types::{MemePath, RawFile};

#[cfg_attr(any(test, feature = "testing"), ::mockall::automock)]
pub trait FileManager: Send + Sync {
    fn upload(
        &self,
        path: &MemePath,
        file: &RawFile,
    ) -> impl Future<Output = Result<(), UploadError>> + Send;

    fn upsert(
        &self,
        path: &MemePath,
        file: &RawFile,
    ) -> impl Future<Output = Result<UpsertStatus, UpsertError>> + Send;

    fn delete(
        &self,
        path: &MemePath,
    ) -> impl Future<Output = Result<DeleteStatus, DeleteError>> + Send;

    fn download(
        &self,
        path: &MemePath,
    ) -> impl Future<Output = Result<RawFile, DownloadError>> + Send;
}

#[derive(Debug, ::thiserror::Error)]
#[error(transparent)]
pub enum FileManagerError {
    Upload(#[from] UploadError),
    Upsert(#[from] UpsertError),
    Delete(#[from] DeleteError),
    Download(#[from] DownloadError),
}

#[derive(Debug, ::thiserror::Error)]
pub enum UploadError {
    #[error("path exists: {path}")]
    FileExists { path: MemePath },
    #[error("unknown upload error: {0}")]
    Unknown(Box<dyn std::error::Error + Send + Sync>),
}

#[derive(Debug, ::thiserror::Error)]
pub enum UpsertError {
    #[error("unknown upsert error: {0}")]
    Unknown(Box<dyn std::error::Error + Send + Sync>),
}

#[derive(Debug, ::thiserror::Error)]
pub enum DeleteError {
    #[error("unknown delete error: {0}")]
    Unknown(Box<dyn std::error::Error + Send + Sync>),
}

#[derive(Debug, ::thiserror::Error)]
pub enum DownloadError {
    #[error("path not found: {path}")]
    FileNotFound { path: MemePath },
    #[error("unknown download error: {0}")]
    Unknown(Box<dyn std::error::Error + Send + Sync>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeleteStatus {
    Deleted,
    NotFound,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpsertStatus {
    Added,
    Updated,
}

#[cfg(any(test, feature = "testing"))]
pub async fn test_file_manager<T: FileManager>(
    mgr: &T,
) -> Result<(), FileManagerError> {
    let init_path = MemePath::from("init-meme.jpg");
    let init_file = RawFile::from_iter([1, 2, 3]);
    let update_path = MemePath::from("update-meme.jpg");
    let updated_file = RawFile::from_iter([4, 5, 6]);

    mgr.upload(&init_path, &init_file).await?;

    let Err(upload_error) = mgr.upload(&init_path, &init_file).await else {
        panic!("expected upload to fail when file already exists");
    };
    assert!(matches!(
        upload_error,
        UploadError::FileExists { path } if path == init_path
    ));

    let found = mgr.download(&init_path).await?;
    assert_eq!(&found, &init_file);

    let status = mgr.upsert(&init_path, &updated_file).await?;
    assert_eq!(status, UpsertStatus::Updated);

    let status = mgr.upsert(&update_path, &updated_file).await?;
    assert_eq!(status, UpsertStatus::Added);

    let found = mgr.download(&update_path).await?;
    assert_eq!(&found, &updated_file);

    let status = mgr.delete(&update_path).await?;
    assert_eq!(status, DeleteStatus::Deleted);

    let status = mgr.delete(&update_path).await?;
    assert_eq!(status, DeleteStatus::NotFound);

    let Err(err) = mgr.download(&update_path).await else {
        panic!("expected download to fail when file not found");
    };
    assert!(matches!(
        err,
        DownloadError::FileNotFound { path } if path == update_path
    ));

    Ok(())
}
