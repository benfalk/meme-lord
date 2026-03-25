use crate::port::event_publisher::MockEventPublisher;
use crate::port::file_manager::MockFileManager;
use crate::port::id_generator::MockIdGenerator;
use crate::port::meme_repo::MockMemeRepo;
use crate::sevice::Env;

pub type MockEnv =
    Env<MockFileManager, MockMemeRepo, MockIdGenerator, MockEventPublisher>;

#[rstest::fixture]
pub fn mock_env() -> MockEnv {
    Env {
        file_manager: MockFileManager::new(),
        meme_repo: MockMemeRepo::new(),
        id_generator: MockIdGenerator::new(),
        event_publisher: MockEventPublisher::new(),
    }
}
