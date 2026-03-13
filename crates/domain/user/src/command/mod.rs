pub mod change_password;
pub mod sign_in;
pub mod sign_up;

/// Cuts down on the amount of import statments needed for each command
/// by pulling in all of the required traits and environment
mod prelude {
    pub use crate::port::event_publisher::EventPublisher;
    pub use crate::port::password_hasher::PasswordHasher;
    pub use crate::port::user_repo::UserRepo;
    pub use crate::service::{Command, Env};
}
