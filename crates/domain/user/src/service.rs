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

pub trait EnvExt: Send + Sync {
    type UR: UserRepo;
    type PH: PasswordHasher;
    type EP: EventPublisher;

    fn user_repo(&self) -> &Self::UR;
    fn password_hasher(&self) -> &Self::PH;
    fn event_publisher(&self) -> &Self::EP;
}

impl<UR, PH, EP> EnvExt for Env<UR, PH, EP>
where
    UR: UserRepo,
    PH: PasswordHasher,
    EP: EventPublisher,
{
    type UR = UR;
    type PH = PH;
    type EP = EP;

    fn user_repo(&self) -> &Self::UR {
        &self.user_repo
    }

    fn password_hasher(&self) -> &Self::PH {
        &self.password_hasher
    }

    fn event_publisher(&self) -> &Self::EP {
        &self.event_publisher
    }
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
