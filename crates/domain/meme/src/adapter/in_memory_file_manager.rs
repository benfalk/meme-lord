use crate::port::file_uploader::*;
use crate::types::{MemePath, RawFile};
use ::std::collections::HashMap;
use ::std::sync::Arc;
use ::tokio::sync::RwLock;

#[derive(Debug, Clone, Default)]
pub struct InMemoryFileManager {
    data: Arc<RwLock<HashMap<MemePath, RawFile>>>,
}

impl FileManager for InMemoryFileManager {
    async fn upload(
        &self,
        path: &MemePath,
        file: &RawFile,
    ) -> Result<(), UploadError> {
        let mut write = self.data.write().await;

        if write.contains_key(path) {
            return Err(UploadError::FileExists { path: path.clone() });
        }

        write.insert(path.clone(), file.clone());

        Ok(())
    }

    async fn upsert(
        &self,
        path: &MemePath,
        file: &RawFile,
    ) -> Result<UpsertStatus, UpsertError> {
        let mut write = self.data.write().await;

        match write.insert(path.clone(), file.clone()) {
            Some(_) => Ok(UpsertStatus::Updated),
            None => Ok(UpsertStatus::Added),
        }
    }

    async fn delete(&self, path: &MemePath) -> Result<DeleteStatus, DeleteError> {
        let mut write = self.data.write().await;

        match write.remove_entry(path) {
            Some(_) => Ok(DeleteStatus::Deleted),
            None => Ok(DeleteStatus::NotFound),
        }
    }

    async fn download(&self, path: &MemePath) -> Result<RawFile, DownloadError> {
        let read = self.data.read().await;

        match read.get(path) {
            Some(file) => Ok(file.clone()),
            None => Err(DownloadError::FileNotFound { path: path.clone() }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn passes_port_tests() {
        let manager = InMemoryFileManager::default();
        crate::port::file_uploader::test_file_manager(&manager)
            .await
            .expect("to pass all of the filer manager tests");
    }
}
