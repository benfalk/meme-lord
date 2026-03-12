use crate::port::password_hasher::PasswordHasher;
use crate::port::user_repo::UserRepo;
use ::std::sync::Arc;

pub struct Service<UR, PH>
where
    UR: UserRepo,
    PH: PasswordHasher,
{
    env: Arc<Env<UR, PH>>,
}

pub struct Env<UR, PH>
where
    UR: UserRepo,
    PH: PasswordHasher,
{
    pub(crate) user_repo: UR,
    pub(crate) password_hasher: PH,
}

#[bon::bon]
impl<UR, PH> Service<UR, PH>
where
    UR: UserRepo,
    PH: PasswordHasher,
{
    #[builder]
    pub fn new(user_repo: UR, password_hasher: PH) -> Self {
        Self {
            env: Arc::new(Env {
                user_repo,
                password_hasher,
            }),
        }
    }

    pub async fn run_command<C>(&self, cmd: C) -> Result<C::Value, C::Error>
    where
        C: Command<UR, PH>,
    {
        cmd.exec(self.env.as_ref()).await
    }

    pub async fn run_query<Q>(&self, query: Q) -> Result<Q::Value, Q::Error>
    where
        Q: Query<UR, PH>,
    {
        query.query(self.env.as_ref()).await
    }
}

pub trait Command<UR, PH>: ::std::fmt::Debug
where
    UR: UserRepo,
    PH: PasswordHasher,
{
    type Error: ::std::error::Error + Send + Sync;
    type Value;

    fn exec(
        self,
        env: &Env<UR, PH>,
    ) -> impl Future<Output = Result<Self::Value, Self::Error>> + Send;
}

pub trait Query<UR, PH>: ::std::fmt::Debug
where
    UR: UserRepo,
    PH: PasswordHasher,
{
    type Error: ::std::error::Error + Send + Sync;
    type Value;

    fn query(
        self,
        env: &Env<UR, PH>,
    ) -> impl Future<Output = Result<Self::Value, Self::Error>> + Send;
}
