use crate::types::{ByteSize, MemeCaption, MemeId, MemePath};
use ::identity::UserId;

/// This is a reprsentation of a `meme` in the system. It contains all the
/// necessary information about a meme, such as its id, owner, url and file
/// size.  Later on we can add more fields to this struct **IF** it makes
/// sense to this domain to do so.
///
/// Lets imagine that there are some extra-greedy capatalists out there who
/// want to make money by selling information gleened from the memes, such
/// as number of views, number of likes, etc. We can add those fields to a
/// **NEW** entity in a new domain named `meme-analytics`.
///
/// The core of what we want to do here is pretend we're in a utopia where
/// we only have the information we need to do our work, and nothing more.
/// Do we need to really track who has accessed the meme?  Who has favorited
/// it?  Those are complicated questions that are sure to evolve over time.
/// All we care about here is that a user can create a meme which stores it
/// somewhere it can be retrieved later for either sharing or viewing from
/// some location.
///
/// ---
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Meme {
    pub id: MemeId,
    pub owner_id: UserId,
    pub path: MemePath,
    pub caption: Option<MemeCaption>,
    pub file_size: ByteSize,
}

#[cfg(any(test, feature = "testing"))]
mod impl_fake {
    use super::*;
    use ::fake::{Dummy, Fake, Faker, Rng};

    impl Dummy<Faker> for Meme {
        fn dummy_with_rng<R: Rng + ?Sized>(_config: &Faker, rng: &mut R) -> Self {
            Self {
                id: Faker.fake_with_rng(rng),
                owner_id: Faker.fake_with_rng(rng),
                path: Faker.fake_with_rng(rng),
                file_size: ByteSize(rng.random_range(8000..250_000_000)),
                caption: Faker.fake_with_rng(rng),
            }
        }
    }
}
