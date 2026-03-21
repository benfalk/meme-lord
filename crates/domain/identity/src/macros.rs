macro_rules! build_identity {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name(::uuid::Uuid);

        ::paste::paste! {
            #[derive(Debug, ::thiserror::Error)]
            #[error("invalid UUID version: {0}")]
            pub struct [<$name VersionError>](pub usize);
        }

        ::paste::paste! {
            #[derive(Debug, ::thiserror::Error)]
            pub enum [<$name ParseError>] {
                #[error("invalid UUID version: {0}")]
                InvalidVersion(usize),

                #[error("invalid UUID: {0}")]
                Uuid(#[from] ::uuid::Error),
            }
        }

        impl $name {
            pub fn generate() -> Self {
                Self(::uuid::Uuid::now_v7())
            }

            pub fn into_inner(self) -> ::uuid::Uuid {
                self.0
            }

            pub fn created_at(&self) -> ::jiff::Timestamp {
                let bytes = self.0.as_bytes();
                let millis: u64 = (bytes[0] as u64) << 40
                    | (bytes[1] as u64) << 32
                    | (bytes[2] as u64) << 24
                    | (bytes[3] as u64) << 16
                    | (bytes[4] as u64) << 8
                    | (bytes[5] as u64);
                let millis = i64::try_from(millis).unwrap_or_default();
                ::jiff::Timestamp::from_millisecond(millis).unwrap_or({
                    if millis < 0 {
                        ::jiff::Timestamp::MIN
                    } else {
                        ::jiff::Timestamp::MAX
                    }
                })
            }
        }

        impl TryFrom<::uuid::Uuid> for $name {
            type Error = ::paste::paste! { [<$name VersionError>] };

            fn try_from(value: ::uuid::Uuid) -> Result<Self, Self::Error> {
                if value.get_version_num() == 7 {
                    Ok(Self(value))
                } else {
                    Err(::paste::paste! {
                        [<$name VersionError>](value.get_version_num())
                    })
                }
            }
        }

        impl ::std::str::FromStr for $name {
            type Err = ::paste::paste! { [<$name ParseError>] };

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let uuid = s.parse::<::uuid::Uuid>()?;
                Self::try_from(uuid).map_err(Into::into)
            }
        }

        impl ::std::borrow::Borrow<::uuid::Uuid> for $name {
            fn borrow(&self) -> &::uuid::Uuid {
                &self.0
            }
        }

        impl ::std::cmp::PartialEq<::uuid::Uuid> for $name {
            fn eq(&self, other: &::uuid::Uuid) -> bool {
                self.0 == *other
            }
        }

        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                self.0.fmt(f)
            }
        }

        ::paste::paste! {
            impl From<[<$name VersionError>]> for [<$name ParseError>] {
                fn from(value: [<$name VersionError>]) -> Self {
                    Self::InvalidVersion(value.0)
                }
            }
        }

        #[cfg(any(test, feature = "testing"))]
        impl ::fake::Dummy<::fake::Faker> for $name {
            fn dummy_with_rng<R: ::fake::Rng + ?Sized>(
                _: &::fake::Faker,
                _: &mut R,
            ) -> Self {
                Self::generate()
            }
        }
    };
}

pub(crate) use build_identity;

#[cfg(test)]
mod tests {
    use ::jiff::{Timestamp, ToSpan};
    use ::uuid::Uuid;

    build_identity!(TestId);

    #[rstest::rstest]
    fn can_generate_a_new_id() {
        let first = TestId::generate();
        let second = TestId::generate();
        assert_ne!(first, second);
    }

    #[rstest::rstest]
    fn can_build_from_a_v7_uuid() {
        let v7_uuid = Uuid::now_v7();
        let test_id = TestId::try_from(v7_uuid).expect("to build from v7 UUID");
        assert_eq!(test_id, v7_uuid);
        assert_eq!(test_id.into_inner(), v7_uuid);
    }

    #[rstest::rstest]
    fn can_parse_from_a_v7_uuid_string() {
        let v7_uuid = Uuid::now_v7();
        let v7_uuid_str = v7_uuid.to_string();
        let test_id = v7_uuid_str
            .parse::<TestId>()
            .expect("to parse from v7 UUID string");
        assert_eq!(test_id, v7_uuid);
    }

    #[rstest::rstest]
    fn fails_to_build_from_non_v7_uuid() {
        let v4_uuid = Uuid::nil();
        let err = TestId::try_from(v4_uuid)
            .expect_err("to fail building from non-v7 UUID");
        assert_eq!(err.0, 0);
    }

    #[rstest::rstest]
    fn can_get_created_at_timestamp() {
        let test_id = TestId::generate();
        let created_at = test_id.created_at();
        let now = Timestamp::now();
        assert!(created_at <= now, "created_at should be in the past");
        assert!(
            created_at > now - 2.milliseconds(),
            "created_at should be within the last 2 milliseconds"
        );
    }
}
