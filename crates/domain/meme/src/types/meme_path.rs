#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MemePath(String);

impl MemePath {
    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }
}

mod impls {
    use super::*;

    impl<T: Into<String>> From<T> for MemePath {
        fn from(value: T) -> Self {
            Self(value.into())
        }
    }

    impl<T: AsRef<str>> ::std::cmp::PartialEq<T> for MemePath {
        fn eq(&self, other: &T) -> bool {
            self.0.as_str() == other.as_ref()
        }
    }

    impl ::std::borrow::Borrow<str> for MemePath {
        fn borrow(&self) -> &str {
            self.0.as_ref()
        }
    }

    impl ::std::fmt::Display for MemePath {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            self.0.fmt(f)
        }
    }
}

#[cfg(any(test, feature = "testing"))]
mod impl_fake {
    use super::*;
    use ::fake::{Dummy, Fake, Faker, Rng, faker};

    impl Dummy<Faker> for MemePath {
        fn dummy_with_rng<R: Rng + ?Sized>(_: &Faker, rng: &mut R) -> Self {
            let path: String = faker::filesystem::en::FilePath().fake_with_rng(rng);
            Self::from(path)
        }
    }
}
