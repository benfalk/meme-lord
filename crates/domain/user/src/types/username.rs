use ::std::sync::Arc;

/// String value a user supplies to identify themselves.
///
/// This domain ensures that each username is unique across the system, and
/// that it adheres to any constraints (e.g., length, allowed characters).
///
/// ---
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Username(Arc<str>);

impl Username {
    pub fn new(username: impl Into<Arc<str>>) -> Self {
        Self(username.into())
    }

    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }
}

mod trait_impls {
    use super::*;

    impl<T: Into<Arc<str>>> From<T> for Username {
        fn from(value: T) -> Self {
            Self::new(value)
        }
    }

    impl<T: AsRef<str>> ::std::cmp::PartialEq<T> for Username {
        fn eq(&self, other: &T) -> bool {
            self.0.as_ref() == other.as_ref()
        }
    }

    impl ::std::ops::Deref for Username {
        type Target = str;

        fn deref(&self) -> &Self::Target {
            self.0.as_ref()
        }
    }

    impl ::std::borrow::Borrow<str> for Username {
        fn borrow(&self) -> &str {
            self.0.as_ref()
        }
    }

    impl ::std::fmt::Display for Username {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            self.0.fmt(f)
        }
    }
}

#[cfg(any(test, feature = "testing"))]
mod impl_fake {
    use super::*;
    use ::fake::{Dummy, Fake, Faker, Rng, faker};

    impl Dummy<Faker> for Username {
        fn dummy_with_rng<R: Rng + ?Sized>(_: &Faker, rng: &mut R) -> Self {
            let username: String =
                faker::internet::en::Username().fake_with_rng(rng);
            Self::new(username)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::std::collections::HashSet;

    #[rstest::rstest]
    fn comparing_with_strings() {
        let username = Username::new("alice");
        let string_name = String::from("alice");
        assert_eq!(username, "alice");
        assert_eq!(username, &string_name);
        assert_eq!(username, username.as_str());
    }

    #[rstest::rstest]
    fn can_be_used_as_hash_map_key() {
        let mut set = HashSet::new();
        set.insert(Username::new("alice"));
        assert!(set.contains("alice"));
    }

    #[rstest::rstest]
    fn can_be_displayed() {
        let username = Username::new("alice");
        let s = format!("{}", username);
        assert_eq!(s, "alice");
    }

    #[rstest::rstest]
    fn can_do_string_operations() {
        let username = Username::new("alice");
        assert_eq!(username.len(), 5);
        assert!(username.starts_with("al"));
        assert!(username.ends_with("ce"));
    }
}
