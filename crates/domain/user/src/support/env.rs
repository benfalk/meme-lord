use crate::port::password_hasher::MockPasswordHasher;
use crate::port::user_repo::MockUserRepo;
use crate::service::Env;

pub type MockEnv = Env<MockUserRepo, MockPasswordHasher>;

#[rstest::fixture]
pub fn mock_env() -> MockEnv {
    Env {
        user_repo: MockUserRepo::new(),
        password_hasher: MockPasswordHasher::new(),
    }
}
