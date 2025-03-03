use async_trait::async_trait;
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
    async fn get_user(
        &self,
        user_id: Option<Uuid>,
        name: Option<&str>,
        email: Option<&str>,
        token: Option<&str>,
    ) -> Result<Option<User>> {
        let mut user: Option<User> = None;

        if let Some(user_id) = user_id {
            user = sqlx::query_as::<_, User>(
                r#"SELECT id, name, email, password, verified, created_at, updated_at, verification_token, token_expires_at, role as "role: UserRole" FROM users WHERE id = $1"#,
            )
            .bind(user_id)
            .fetch_optional(&self.pool).await?;
        } else if let Some(name) = name {
            user = sqlx::query_as::<_, User>(
                r#"SELECT id, name, email, password, verified, created_at, updated_at, verification_token, token_expires_at, role as "role: UserRole" FROM users WHERE name = $1"#,
            )
            .bind(name)
            .fetch_optional(&self.pool).await?;
        } else if let Some(email) = email {
            user = sqlx::query_as::<_, User>(
                r#"SELECT id, name, email, password, verified, created_at, updated_at, verification_token, token_expires_at, role as "role: UserRole" FROM users WHERE email = $1"#,
            )
            .bind(email)
            .fetch_optional(&self.pool).await?;
        } else if let Some(token) = token {
            user = sqlx::query_as::<_, User>(
                r#"
                SELECT id, name, email, password, verified, created_at, updated_at, verification_token, token_expires_at, role as "role: UserRole"
                FROM users
                WHERE verification_token = $1"#,
            )
            .bind(token)
            .fetch_optional(&self.pool)
            .await?;
        }

        Ok(user)
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
            RETURNING id, name, email, password, verified, created_at, updated_at, verification_token, token_expires_at, role as "role: UserRole"
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
