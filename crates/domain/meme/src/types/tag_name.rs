use ::std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TagName(Arc<str>);

impl TagName {
    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }
}

mod impls {
    use super::*;

    impl<T: Into<Arc<str>>> From<T> for TagName {
        fn from(value: T) -> Self {
            Self(value.into())
        }
    }

    impl<T: AsRef<str>> ::std::cmp::PartialEq<T> for TagName {
        fn eq(&self, other: &T) -> bool {
            self.0.as_ref() == other.as_ref()
        }
    }

    impl ::std::borrow::Borrow<str> for TagName {
        fn borrow(&self) -> &str {
            self.0.as_ref()
        }
    }

    impl ::std::fmt::Display for TagName {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            self.0.fmt(f)
        }
    }
}

#[cfg(any(test, feature = "testing"))]
mod impl_fake {
    use super::*;
    use ::fake::{Dummy, Fake, Faker, RngExt, faker};

    impl Dummy<Faker> for TagName {
        fn dummy_with_rng<R: RngExt + ?Sized>(_: &Faker, rng: &mut R) -> Self {
            let tag_name: String = faker::lorem::en::Word().fake_with_rng(rng);
            Self::from(tag_name)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest::rstest]
    fn string_like_behaviour() {
        let caption = TagName::from("tag");
        assert_eq!(caption.as_str(), "tag");
        assert_eq!(caption, "tag");
        assert_eq!(caption.to_string(), "tag");
    }
}
