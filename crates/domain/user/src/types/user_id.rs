use crate::types::Timestamp;
use ::uuid::Uuid;

/// UUID the system uses to identify users.
///
/// We use UUIDv7, which is a time-ordered UUID that provides better
/// performance for databases and distributed systems.  This also allows
/// us to know **when** a user was created just by looking at their ID.
///
/// ---
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ::derive_more::Display, Hash,
)]
pub struct UserId(Uuid);

#[derive(Debug, ::thiserror::Error)]
pub enum UserIdError {
    #[error(transparent)]
    Parse(#[from] UserIdParseError),

    #[error("invalid UUID: {0}")]
    InvalidVersion(usize),
}

#[derive(Debug, ::thiserror::Error)]
pub enum UserIdParseError {
    #[error("invalid UUID version: {0}")]
    InvalidVersion(usize),

    #[error("invalid UUID: {0}")]
    Uuid(#[from] ::uuid::Error),
}

#[derive(Debug, ::thiserror::Error)]
#[error("invalid UUID version: {0}")]
pub struct UserIdVersionError(pub usize);

impl UserId {
    pub const EXPECTED_UUID_VERSION: usize = 7;

    pub fn generate() -> Self {
        Self(Uuid::now_v7())
    }

    pub fn into_inner(self) -> Uuid {
        self.0
    }

    pub fn parse_str(s: &str) -> Result<Self, UserIdParseError> {
        s.parse()
    }

    pub fn created_at(&self) -> Timestamp {
        let bytes = self.0.as_bytes();
        let millis: u64 = (bytes[0] as u64) << 40
            | (bytes[1] as u64) << 32
            | (bytes[2] as u64) << 24
            | (bytes[3] as u64) << 16
            | (bytes[4] as u64) << 8
            | (bytes[5] as u64);
        let millis = i64::try_from(millis).unwrap_or_default();
        Timestamp::from_millisecond(millis).unwrap_or({
            if millis < 0 {
                Timestamp::MIN
            } else {
                Timestamp::MAX
            }
        })
    }
}

#[cfg(any(test, feature = "testing"))]
mod impl_fake {
    use super::*;
    use ::fake::{Dummy, Faker, Rng};

    impl Dummy<Faker> for UserId {
        fn dummy_with_rng<R: Rng + ?Sized>(_: &Faker, _: &mut R) -> Self {
            Self::generate()
        }
    }
}

mod trait_impls {
    use super::*;

    impl ::std::borrow::Borrow<Uuid> for UserId {
        fn borrow(&self) -> &Uuid {
            &self.0
        }
    }

    impl TryFrom<Uuid> for UserId {
        type Error = UserIdVersionError;

        fn try_from(value: Uuid) -> Result<Self, Self::Error> {
            let version = value.get_version_num();
            if version != Self::EXPECTED_UUID_VERSION {
                return Err(UserIdVersionError(version));
            }
            Ok(Self(value))
        }
    }

    impl ::std::str::FromStr for UserId {
        type Err = UserIdParseError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let uuid = Uuid::parse_str(s)?;
            Self::try_from(uuid).map_err(UserIdParseError::from)
        }
    }

    impl ::std::cmp::PartialEq<Uuid> for UserId {
        fn eq(&self, other: &Uuid) -> bool {
            self.0 == *other
        }
    }

    impl From<UserIdVersionError> for UserIdParseError {
        fn from(value: UserIdVersionError) -> Self {
            Self::InvalidVersion(value.0)
        }
    }

    impl From<UserIdVersionError> for UserIdError {
        fn from(value: UserIdVersionError) -> Self {
            Self::InvalidVersion(value.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::jiff::ToSpan;

    #[rstest::rstest]
    fn try_from_uuid_with_invalid_version() {
        let uuid = Uuid::nil();
        let result = UserId::try_from(uuid);
        assert!(matches!(result, Err(UserIdVersionError(0))));
    }

    #[rstest::rstest]
    fn try_from_uuid_with_valid_version() {
        let uuid = Uuid::now_v7();
        let result = UserId::try_from(uuid).expect("a valid UserId");
        assert_eq!(result, uuid);
    }

    #[rstest::rstest]
    fn parse_str_with_invalid_uuid() {
        let result = "not-a-uuid".parse::<UserId>();
        assert!(matches!(result, Err(UserIdParseError::Uuid(_))));
    }

    #[rstest::rstest]
    fn parse_str_with_invalid_uuid_version() {
        let uuid = Uuid::nil();
        let result = uuid.to_string().parse::<UserId>();
        assert!(matches!(result, Err(UserIdParseError::InvalidVersion(0))));
    }

    #[rstest::rstest]
    fn can_get_creation_time_from_user_id() {
        let user_id = UserId::generate();
        let created_at = user_id.created_at();
        let now = Timestamp::now();
        assert!(created_at <= now, "created_at should be in the past");
        assert!(
            created_at > now - 2.milliseconds(),
            "created_at should be recent"
        );
    }
}
