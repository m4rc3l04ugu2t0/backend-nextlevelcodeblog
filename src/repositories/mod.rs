use sqlx::PgPool;

pub mod posts_repo;
pub mod user_repo;

#[derive(Clone)]
pub struct PostgresRepo {
    pool: PgPool,
}

impl PostgresRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
