#[derive(::derive_more::Debug, Clone, PartialEq, Eq)]
#[cfg_attr(any(test, feature = "testing"), derive(::fake::Dummy))]
pub struct PasswordHash(Vec<u8>);

impl PasswordHash {
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

mod impls {
    use super::*;

    impl From<Vec<u8>> for PasswordHash {
        fn from(value: Vec<u8>) -> Self {
            Self(value)
        }
    }

    impl From<String> for PasswordHash {
        fn from(value: String) -> Self {
            Self(value.into_bytes())
        }
    }
}
