/// # This type represents a user's password
///
/// ## Security Considerations
///
/// Designed to prevent accidental logging of the password by implementing
/// custom `Debug` and `Display` traits that do not reveal the actual
/// password value. It also does not allow cloning to avoid accidental
/// copying of the password in memory. Once a `Password` is created, it
/// cannot be modified or revealed, ensuring that the password remains
/// secure throughout its lifecycle.
///
/// ---
#[derive(::derive_more::Debug, ::derive_more::Display)]
#[debug("Password(*********)")]
#[display("*********")]
pub struct Password(String);

mod impls {
    use super::*;

    impl<T: Into<String>> From<T> for Password {
        fn from(value: T) -> Self {
            Self(value.into())
        }
    }

    impl AsRef<[u8]> for Password {
        fn as_ref(&self) -> &[u8] {
            self.0.as_bytes()
        }
    }
}

#[cfg(any(test, feature = "testing"))]
mod impl_fake {
    use super::*;
    use ::fake::{Dummy, Fake, Faker, Rng, faker};

    impl Dummy<Faker> for Password {
        fn dummy_with_rng<R: Rng + ?Sized>(_: &Faker, rng: &mut R) -> Self {
            let username: String =
                faker::internet::en::Password(18..25).fake_with_rng(rng);
            Self::from(username)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest::rstest]
    fn debug_does_not_reveal_password() {
        let password = Password::from("secret");
        let debug_output = format!("{:?}", password);
        assert_eq!(debug_output, "Password(*********)");
    }

    #[rstest::rstest]
    fn display_does_not_reveal_password() {
        let password = Password::from("secret");
        let display_output = format!("{}", password);
        assert_eq!(display_output, "*********");
    }
}
