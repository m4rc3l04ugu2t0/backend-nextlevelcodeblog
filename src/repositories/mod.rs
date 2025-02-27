use sqlx::PgPool;

pub mod auth_repo;
pub mod news_post_repo;
pub mod user_repo;
pub mod videos_repo;

#[derive(Clone)]
pub struct PostgresRepo {
    pool: PgPool,
}

impl PostgresRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
