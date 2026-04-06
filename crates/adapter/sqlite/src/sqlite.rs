use ::sqlx::SqlitePool;
use ::sqlx::migrate::MigrateError;

pub struct Sqlite {
    pub(crate) pool: SqlitePool,
}

impl Sqlite {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn migrate(&self) -> Result<(), MigrateError> {
        sqlx::migrate!("./migrations").run(&self.pool).await
    }

    pub async fn in_memory() -> Self {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        Self { pool }
    }

    pub async fn open(path: &str) -> Result<Self, ::sqlx::Error> {
        let pool = SqlitePool::connect(path).await?;
        Ok(Self { pool })
    }
}
