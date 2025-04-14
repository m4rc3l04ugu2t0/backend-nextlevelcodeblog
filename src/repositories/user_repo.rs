use async_trait::async_trait;
use tracing::{info_span, instrument};
use uuid::Uuid;

use crate::{models::users::User, Result};

use super::PostgresRepo;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn get_user(
        &self,
        user_id: Option<Uuid>,
        name: Option<&str>,
        email: Option<&str>,
        token: Option<&str>,
    ) -> Result<Option<User>>;

    async fn update_password(&self, user_id: Uuid, new_password: &str) -> Result<()>;
    async fn update_username(&self, user_id: Uuid, new_username: &str) -> Result<User>;
    async fn delete_user(&self, user_id: Uuid) -> Result<()>;
}


#[async_trait]
impl UserRepository for PostgresRepo {
    #[instrument(skip(self))]
    async fn get_user(
        &self,
        user_id: Option<Uuid>,
        name: Option<&str>,
        email: Option<&str>,
        token: Option<&str>,
    ) -> Result<Option<User>> {
        let span = info_span!("get_user", user_id = ?user_id, name = ?name, email = ?email, token = ?token);
        let _enter = span.enter();

        if let Some(user_id) = user_id {
            let sql = r#"
                SELECT id, name, email, password, verified, created_at, updated_at, verification_token, token_expires_at, role
                FROM users
                WHERE id = $1
            "#;
            tracing::debug!("Fetching user by ID: {}", user_id);
            let user = sqlx::query_as::<_, User>(sql)
                .bind(user_id)
                .fetch_optional(&self.pool)
                .await?;
            return Ok(user);
        }

        if let Some(name) = name {
            let sql = r#"
                SELECT id, name, email, password, verified, created_at, updated_at, verification_token, token_expires_at, role
                FROM users
                WHERE name = $1
            "#;
            tracing::debug!("Fetching user by name: {}", name);
            let user = sqlx::query_as::<_, User>(sql)
                .bind(name)
                .fetch_optional(&self.pool)
                .await?;
            return Ok(user);
        }

        if let Some(email) = email {
            let sql = r#"
                SELECT id, name, email, password, verified, created_at, updated_at, verification_token, token_expires_at, role
                FROM users
                WHERE email = $1
            "#;
            tracing::debug!("Fetching user by email: {}", email);
            let user = sqlx::query_as::<_, User>(sql)
                .bind(email)
                .fetch_optional(&self.pool)
                .await?;
            return Ok(user);
        }

        if let Some(token) = token {
            let sql = r#"
                SELECT id, name, email, password, verified, created_at, updated_at, verification_token, token_expires_at, role
                FROM users
                WHERE verification_token = $1
            "#;
            tracing::debug!("Fetching user by token: {}", token);
            let user = sqlx::query_as::<_, User>(sql)
                .bind(token)
                .fetch_optional(&self.pool)
                .await?;
            return Ok(user);
        }

        tracing::warn!("Invalid combination of parameters");
        Ok(None)
    }


    async fn update_password(&self, user_id: Uuid, new_password: &str) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE users
            SET password = $1, updated_at = Now()
            WHERE id = $2
            "#,
            new_password,
            user_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn update_username(&self, user_id: Uuid, new_username: &str) -> Result<User> {
        let user = sqlx::query_as::<_, User>(
            r#"
            UPDATE users
            SET name = $1, updated_at = Now()
            WHERE id = $2
            RETURNING id, name, email, password, verified, created_at, updated_at, verification_token, token_expires_at, role"
            "#
        )
        .bind(new_username)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    async fn delete_user(&self, user_id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
