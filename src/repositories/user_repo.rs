use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    models::users::{User, UserRole},
    Result,
};

use super::PostgresRepo;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create_user(
        &self,
        name: String,
        email: String,
        password: String,
        verification_token: String,
        token_expires_at: Option<DateTime<Utc>>,
    ) -> Result<User>;
    async fn find_by_email(&self, email: &str) -> Result<Option<User>>;
    async fn verifed_token(&self, token_expires_at: String) -> Result<()>;
    async fn get_user(
        &self,
        user_id: Option<Uuid>,
        name: Option<&str>,
        email: Option<&str>,
        token: Option<&str>,
    ) -> Result<Option<User>>;
    async fn add_verifed_token(
        &self,
        user_id: Uuid,
        token_expires_at: DateTime<Utc>,
        token: &str,
    ) -> Result<()>;
    async fn update_password(&self, user_id: Uuid, password: &str) -> Result<()>;
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
            user = sqlx::query_as!(
                User,
                r#"SELECT id, name, email, password, verified, created_at, updated_at, verification_token, token_expires_at, role as "role: UserRole" FROM users WHERE id = $1"#,
                user_id
            ).fetch_optional(&self.pool).await?;
        } else if let Some(name) = name {
            user = sqlx::query_as!(
                User,
                r#"SELECT id, name, email, password, verified, created_at, updated_at, verification_token, token_expires_at, role as "role: UserRole" FROM users WHERE name = $1"#,
                name
            ).fetch_optional(&self.pool).await?;
        } else if let Some(email) = email {
            user = sqlx::query_as!(
                User,
                r#"SELECT id, name, email, password, verified, created_at, updated_at, verification_token, token_expires_at, role as "role: UserRole" FROM users WHERE email = $1"#,
                email
            ).fetch_optional(&self.pool).await?;
        } else if let Some(token) = token {
            user = sqlx::query_as!(
                User,
                r#"
                SELECT id, name, email, password, verified, created_at, updated_at, verification_token, token_expires_at, role as "role: UserRole"
                FROM users
                WHERE verification_token = $1"#,
                token
            )
            .fetch_optional(&self.pool)
            .await?;
        }

        Ok(user)
    }

    async fn create_user(
        &self,
        name: String,
        email: String,
        password: String,
        verification_token: String,
        token_expires_at: Option<DateTime<Utc>>, // Make sure this is Option
    ) -> Result<User> {
        let user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (id, name, email, password, verification_token, token_expires_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, name, email, password, verified, created_at, updated_at, verification_token, token_expires_at, role as "role: UserRole"
            "#,
            Uuid::now_v7(),
            name.into(),
            email.into(),
            password.into(),
            verification_token.into(),
            token_expires_at // This can now be an Option<DateTime<Utc>>
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT id, name, email, password, verified, created_at, updated_at, verification_token, token_expires_at, role as "role: UserRole"
            FROM users
            WHERE email = $1
            "#,
            email
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    async fn verifed_token(&self, token: String) -> Result<()> {
        let _ = sqlx::query!(
            r#"
            UPDATE users
            SET verified = true,
                updated_at = Now(),
                verification_token = NULL,
                token_expires_at = NULL
            WHERE verification_token = $1
            "#,
            &token
        )
        .execute(&self.pool)
        .await;

        Ok(())
    }

    async fn add_verifed_token(
        &self,
        user_id: Uuid,
        token_expires_at: DateTime<Utc>,
        token: &str,
    ) -> Result<()> {
        let _ = sqlx::query!(
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
        .await;

        Ok(())
    }

    async fn update_password(&self, user_id: Uuid, password: &str) -> Result<()> {
        let _ = sqlx::query!(
            r#"
            UPDATE users
            SET password = $1
            WHERE id = $2
            "#,
            password,
            user_id
        )
        .execute(&self.pool)
        .await;

        Ok(())
    }
}
