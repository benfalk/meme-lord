use crate::types::{Password, PasswordHash};

#[derive(Debug, ::thiserror::Error)]
#[error("failed to hash password")]
pub struct HashError {
    pub source: Box<dyn std::error::Error + Send + Sync>,
}

#[cfg_attr(any(test, feature = "testing"), ::mockall::automock)]
pub trait PasswordHasher: Send + Sync {
    fn hash(
        &self,
        password: &Password,
    ) -> impl Future<Output = Result<PasswordHash, HashError>> + Send;
    fn verify(
        &self,
        password: &Password,
        hash: &PasswordHash,
    ) -> impl Future<Output = Result<bool, HashError>> + Send;
}

#[cfg(any(test, feature = "testing"))]
pub async fn test_adapter<PH: PasswordHasher>(
    adapter: &PH,
) -> Result<(), HashError> {
    let password = Password::from("my_secret_password");
    let another_password = Password::from("another_secret_password");
    let hash = adapter.hash(&password).await?;
    assert!(adapter.verify(&password, &hash).await?);
    assert!(!adapter.verify(&another_password, &hash).await?);
    Ok(())
}
