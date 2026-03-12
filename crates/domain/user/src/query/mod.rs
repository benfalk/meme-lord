pub mod find_by_id;
pub mod find_by_username;

/// Cuts down on the amount of import statments needed for each command
/// by pulling in all of the required traits and environment
mod prelude {
    pub use crate::port::password_hasher::PasswordHasher;
    pub use crate::port::user_repo::UserRepo;
    pub use crate::service::{Env, Query};
}
