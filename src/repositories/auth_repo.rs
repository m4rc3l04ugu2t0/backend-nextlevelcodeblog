use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{models::users::User, Result};

use super::PostgresRepo;

#[async_trait]
pub trait AuthRepository: Send + Sync {
    async fn create_user(
        &self,
        name: String,
        email: String,
        password: String,
        verification_token: String,
        token_expires_at: Option<DateTime<Utc>>,
    ) -> Result<User>;
    async fn add_verifed_token(
        &self,
        user_id: Uuid,
        token_expires_at: DateTime<Utc>,
        token: &str,
    ) -> Result<()>;

    async fn verifed_token(&self, token: &str) -> Result<()>;
}

#[async_trait]
impl AuthRepository for PostgresRepo {
    async fn create_user(
        &self,
        name: String,
        email: String,
        password: String,
        verification_token: String,
        token_expires_at: Option<DateTime<Utc>>,
    ) -> Result<User> {
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (id, name, email, password, verification_token, token_expires_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, name, email, password, verified, created_at, updated_at, verification_token, token_expires_at, role as "role: UserRole"
            "#,
        )
        .bind(Uuid::now_v7())
        .bind(name)
        .bind(email)
        .bind(password)
        .bind(verification_token)
        .bind(token_expires_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    async fn add_verifed_token(
        &self,
        user_id: Uuid,
        token_expires_at: DateTime<Utc>,
        token: &str,
    ) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE users
            SET verification_token = $1,
                token_expires_at = $2
            WHERE id = $3
            "#,
            token,
            token_expires_at,
            user_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn verifed_token(&self, token: &str) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE users
            SET verified = true,
                updated_at = Now(),
                verification_token = NULL,
                token_expires_at = NULL
            WHERE verification_token = $1
            "#,
            token
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
