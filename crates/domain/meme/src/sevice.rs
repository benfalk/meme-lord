use crate::port::event_publisher::EventPublisher;
use crate::port::file_manager::FileManager;
use crate::port::id_generator::IdGenerator;
use crate::port::meme_repo::MemeRepo;
use ::std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Service<FM, MR, ID, EP>
where
    FM: FileManager,
    MR: MemeRepo,
    ID: IdGenerator,
    EP: EventPublisher,
{
    env: Arc<Env<FM, MR, ID, EP>>,
}

#[derive(Debug, Clone)]
pub struct Env<FM, MR, ID, EP>
where
    FM: FileManager,
    MR: MemeRepo,
    ID: IdGenerator,
    EP: EventPublisher,
{
    pub(crate) file_manager: FM,
    pub(crate) meme_repo: MR,
    pub(crate) id_generator: ID,
    pub(crate) event_publisher: EP,
}

#[bon::bon]
impl<FM, MR, ID, EP> Service<FM, MR, ID, EP>
where
    FM: FileManager,
    MR: MemeRepo,
    ID: IdGenerator,
    EP: EventPublisher,
{
    #[builder]
    pub fn new(
        file_manager: FM,
        meme_repo: MR,
        id_generator: ID,
        event_publisher: EP,
    ) -> Self {
        Self {
            env: Arc::new(Env {
                file_manager,
                meme_repo,
                id_generator,
                event_publisher,
            }),
        }
    }

    pub async fn run_command<C>(&self, cmd: C) -> Result<C::Value, C::Error>
    where
        C: Command<FM, MR, ID, EP>,
    {
        cmd.exec(self.env.as_ref()).await
    }

    pub async fn run_query<Q>(&self, query: Q) -> Result<Q::Value, Q::Error>
    where
        Q: Query<FM, MR, ID, EP>,
    {
        query.query(self.env.as_ref()).await
    }
}

pub trait Command<FM, MR, ID, EP>: ::std::fmt::Debug
where
    FM: FileManager,
    MR: MemeRepo,
    ID: IdGenerator,
    EP: EventPublisher,
{
    type Error: ::std::error::Error + Send + Sync;
    type Value;

    fn exec(
        self,
        env: &Env<FM, MR, ID, EP>,
    ) -> impl Future<Output = Result<Self::Value, Self::Error>> + Send;
}

pub trait Query<FM, MR, ID, EP>: ::std::fmt::Debug
where
    FM: FileManager,
    MR: MemeRepo,
    ID: IdGenerator,
    EP: EventPublisher,
{
    type Error: ::std::error::Error + Send + Sync;
    type Value;

    fn query(
        self,
        env: &Env<FM, MR, ID, EP>,
    ) -> impl Future<Output = Result<Self::Value, Self::Error>> + Send;
}
