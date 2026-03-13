use crate::port::event_publisher::EventPublisher;
use crate::port::password_hasher::PasswordHasher;
use crate::port::user_repo::UserRepo;
use ::std::sync::Arc;

pub struct Service<UR, PH, EP>
where
    UR: UserRepo,
    PH: PasswordHasher,
    EP: EventPublisher,
{
    env: Arc<Env<UR, PH, EP>>,
}

pub struct Env<UR, PH, EP>
where
    UR: UserRepo,
    PH: PasswordHasher,
    EP: EventPublisher,
{
    pub(crate) user_repo: UR,
    pub(crate) password_hasher: PH,
    pub(crate) event_publisher: EP,
}

#[bon::bon]
impl<UR, PH, EP> Service<UR, PH, EP>
where
    UR: UserRepo,
    PH: PasswordHasher,
    EP: EventPublisher,
{
    #[builder]
    pub fn new(user_repo: UR, password_hasher: PH, event_publisher: EP) -> Self {
        Self {
            env: Arc::new(Env {
                user_repo,
                password_hasher,
                event_publisher,
            }),
        }
    }

    pub async fn run_command<C>(&self, cmd: C) -> Result<C::Value, C::Error>
    where
        C: Command<UR, PH, EP>,
    {
        cmd.exec(self.env.as_ref()).await
    }

    pub async fn run_query<Q>(&self, query: Q) -> Result<Q::Value, Q::Error>
    where
        Q: Query<UR, PH, EP>,
    {
        query.query(self.env.as_ref()).await
    }
}

pub trait Command<UR, PH, EP>: ::std::fmt::Debug
where
    UR: UserRepo,
    PH: PasswordHasher,
    EP: EventPublisher,
{
    type Error: ::std::error::Error + Send + Sync;
    type Value;

    fn exec(
        self,
        env: &Env<UR, PH, EP>,
    ) -> impl Future<Output = Result<Self::Value, Self::Error>> + Send;
}

pub trait Query<UR, PH, EP>: ::std::fmt::Debug
where
    UR: UserRepo,
    PH: PasswordHasher,
    EP: EventPublisher,
{
    type Error: ::std::error::Error + Send + Sync;
    type Value;

    fn query(
        self,
        env: &Env<UR, PH, EP>,
    ) -> impl Future<Output = Result<Self::Value, Self::Error>> + Send;
}
