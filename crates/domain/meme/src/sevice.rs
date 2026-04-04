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

pub trait EnvExt: Send + Sync {
    type FM: FileManager;
    type MR: MemeRepo;
    type ID: IdGenerator;
    type EP: EventPublisher;

    fn file_manager(&self) -> &Self::FM;
    fn meme_repo(&self) -> &Self::MR;
    fn id_generator(&self) -> &Self::ID;
    fn event_publisher(&self) -> &Self::EP;
}

impl<FM, MR, ID, EP> EnvExt for Env<FM, MR, ID, EP>
where
    FM: FileManager,
    MR: MemeRepo,
    ID: IdGenerator,
    EP: EventPublisher,
{
    type FM = FM;
    type MR = MR;
    type ID = ID;
    type EP = EP;

    fn file_manager(&self) -> &Self::FM {
        &self.file_manager
    }

    fn meme_repo(&self) -> &Self::MR {
        &self.meme_repo
    }

    fn id_generator(&self) -> &Self::ID {
        &self.id_generator
    }

    fn event_publisher(&self) -> &Self::EP {
        &self.event_publisher
    }
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
        C: Command,
    {
        cmd.exec(self.env.as_ref()).await
    }

    pub async fn run_query<Q>(&self, query: Q) -> Result<Q::Value, Q::Error>
    where
        Q: Query,
    {
        query.query(self.env.as_ref()).await
    }
}

pub trait Command: ::std::fmt::Debug {
    type Error: ::std::error::Error + Send + Sync;
    type Value;

    fn exec(
        self,
        env: &impl EnvExt,
    ) -> impl Future<Output = Result<Self::Value, Self::Error>> + Send;
}

pub trait Query: ::std::fmt::Debug {
    type Error: ::std::error::Error + Send + Sync;
    type Value;

    fn query(
        self,
        env: &impl EnvExt,
    ) -> impl Future<Output = Result<Self::Value, Self::Error>> + Send;
}
