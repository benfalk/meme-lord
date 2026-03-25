use ::std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MemeCaption(Arc<str>);

impl MemeCaption {
    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }
}

mod impls {
    use super::*;

    impl<T: Into<Arc<str>>> From<T> for MemeCaption {
        fn from(value: T) -> Self {
            Self(value.into())
        }
    }

    impl<T: AsRef<str>> ::std::cmp::PartialEq<T> for MemeCaption {
        fn eq(&self, other: &T) -> bool {
            self.0.as_ref() == other.as_ref()
        }
    }

    impl ::std::borrow::Borrow<str> for MemeCaption {
        fn borrow(&self) -> &str {
            self.0.as_ref()
        }
    }

    impl ::std::fmt::Display for MemeCaption {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            self.0.fmt(f)
        }
    }
}

#[cfg(any(test, feature = "testing"))]
mod impl_fake {
    use super::*;
    use ::fake::{Dummy, Fake, Faker, Rng, faker};

    impl Dummy<Faker> for MemeCaption {
        fn dummy_with_rng<R: Rng + ?Sized>(_: &Faker, rng: &mut R) -> Self {
            let caption: String =
                faker::lorem::en::Sentence(1..10).fake_with_rng(rng);
            Self::from(caption)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest::rstest]
    fn string_like_behaviour() {
        let caption = MemeCaption::from("This is a meme caption");
        assert_eq!(caption.as_str(), "This is a meme caption");
        assert_eq!(caption, "This is a meme caption");
        assert_eq!(caption.to_string(), "This is a meme caption");
    }
}
