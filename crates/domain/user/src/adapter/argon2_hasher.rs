use crate::port::password_hasher::*;
use crate::types::{Password, PasswordHash};
use ::argon2::{
    Argon2, PasswordHasher as _, PasswordVerifier as _,
    password_hash::{
        Error as Argon2HashError, PasswordHash as Argon2Hash, SaltString,
        rand_core::OsRng,
    },
};
use ::std::sync::Arc;

#[derive(::derive_more::Debug, Clone, Default)]
#[debug("Argon2Hasher(...)")]
pub struct Argon2Hasher {
    inner: Arc<Inner>,
}

#[derive(Default)]
struct Inner {
    secret: Option<Box<[u8]>>,
}

impl Argon2Hasher {
    pub fn with_secret<T: Into<Box<[u8]>>>(secret: T) -> Result<Self, HashError> {
        let inner = Inner {
            secret: Some(secret.into()),
        };
        inner.argon2()?;
        Ok(Argon2Hasher {
            inner: Arc::new(inner),
        })
    }
}

impl Inner {
    fn argon2(&self) -> Result<Argon2<'_>, HashError> {
        if let Some(secret) = &self.secret {
            Argon2::new_with_secret(
                secret,
                ::argon2::Algorithm::Argon2id,
                ::argon2::Version::V0x13,
                ::argon2::Params::default(),
            )
            .map_err(|err| HashError {
                source: Box::new(err),
            })
        } else {
            Ok(Argon2::default())
        }
    }
}

impl PasswordHasher for Argon2Hasher {
    async fn hash(&self, password: &Password) -> Result<PasswordHash, HashError> {
        let bytes = password.as_ref().to_vec();
        let inner = self.inner.clone();

        let handle = ::tokio::task::spawn_blocking(move || {
            let salt = SaltString::generate(&mut OsRng);
            let argon_hash =
                inner
                    .argon2()?
                    .hash_password(&bytes, &salt)
                    .map_err(|err| HashError {
                        source: Box::new(err),
                    })?;
            Ok(argon_hash.to_string().into())
        });

        handle.await.map_err(|err| HashError {
            source: Box::new(err),
        })?
    }

    async fn verify(
        &self,
        password: &Password,
        hash: &PasswordHash,
    ) -> Result<bool, HashError> {
        let inner = self.inner.clone();
        let bytes = password.as_ref().to_vec();
        let str_hash =
            String::from_utf8(hash.as_bytes().to_vec()).map_err(|err| {
                HashError {
                    source: Box::new(err),
                }
            })?;

        let handle = ::tokio::task::spawn_blocking(move || {
            let argon2_hash =
                Argon2Hash::new(&str_hash).map_err(|err| HashError {
                    source: Box::new(err),
                })?;
            match inner.argon2()?.verify_password(&bytes, &argon2_hash) {
                Ok(()) => Ok(true),
                Err(Argon2HashError::Password) => Ok(false),
                Err(other) => Err(HashError {
                    source: Box::new(other),
                }),
            }
        });

        handle.await.map_err(|err| HashError {
            source: Box::new(err),
        })?
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn works_for_port_with_secret() {
        let argon2 = Argon2Hasher::with_secret("my_secret".as_bytes())
            .expect("an argon2 hasher");
        test_adapter(&argon2)
            .await
            .expect("argon2 should work as a password hasher");
    }

    #[tokio::test]
    async fn works_for_port_without_secret() {
        let argon2 = Argon2Hasher::default();
        test_adapter(&argon2)
            .await
            .expect("argon2 should work as a password hasher");
    }

    #[tokio::test]
    async fn different_secret_hashers_dont_verify() {
        let password = Password::from("my-cool-password");
        let secret = Argon2Hasher::with_secret("secret".as_bytes()).unwrap();
        let same_secret = Argon2Hasher::with_secret("secret".as_bytes()).unwrap();
        let another = Argon2Hasher::with_secret("another".as_bytes()).unwrap();
        let hash = secret.hash(&password).await.unwrap();

        assert!(secret.verify(&password, &hash).await.unwrap());
        assert!(same_secret.verify(&password, &hash).await.unwrap());
        assert!(!another.verify(&password, &hash).await.unwrap());
    }
}
