use ::sqlx::SqlitePool;

pub struct Sqlite {
    pub(crate) pool: SqlitePool,
}

impl Sqlite {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}
