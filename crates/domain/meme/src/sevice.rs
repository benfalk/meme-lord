use crate::port::file_uploader::FileManager;
use crate::port::meme_repo::MemeRepo;
use ::std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Service<FM, MR>
where
    FM: FileManager,
    MR: MemeRepo,
{
    env: Arc<Env<FM, MR>>,
}

#[derive(Debug, Clone)]
pub struct Env<FM, MR>
where
    FM: FileManager,
    MR: MemeRepo,
{
    pub(crate) file_manager: FM,
    pub(crate) meme_repo: MR,
}

#[bon::bon]
impl<FM, MR> Service<FM, MR>
where
    FM: FileManager,
    MR: MemeRepo,
{
    #[builder]
    pub fn new(file_manager: FM, meme_repo: MR) -> Self {
        Self {
            env: Arc::new(Env {
                file_manager,
                meme_repo,
            }),
        }
    }

    pub async fn run_command<C>(&self, cmd: C) -> Result<C::Value, C::Error>
    where
        C: Command<FM, MR>,
    {
        cmd.exec(self.env.as_ref()).await
    }

    pub async fn run_query<Q>(&self, query: Q) -> Result<Q::Value, Q::Error>
    where
        Q: Query<FM, MR>,
    {
        query.query(self.env.as_ref()).await
    }
}

pub trait Command<FM, MR>: ::std::fmt::Debug
where
    FM: FileManager,
    MR: MemeRepo,
{
    type Error: ::std::error::Error + Send + Sync;
    type Value;

    fn exec(
        self,
        env: &Env<FM, MR>,
    ) -> impl Future<Output = Result<Self::Value, Self::Error>> + Send;
}

pub trait Query<FM, MR>: ::std::fmt::Debug
where
    FM: FileManager,
    MR: MemeRepo,
{
    type Error: ::std::error::Error + Send + Sync;
    type Value;

    fn query(
        self,
        env: &Env<FM, MR>,
    ) -> impl Future<Output = Result<Self::Value, Self::Error>> + Send;
}
