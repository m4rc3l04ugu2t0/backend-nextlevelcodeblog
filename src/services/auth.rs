use std::env;

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2, PasswordHash, PasswordVerifier,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    mail::mails::{send_forgot_password_email, send_welcome_email},
    models::users::User,
    repositories::{auth_repo::AuthRepository, user_repo::UserRepository, PostgresRepo},
    Error, Result,
};

#[derive(Clone)]
pub struct AuthService {
    repo: PostgresRepo,
    jwt_secret: String,
    jwt_expiration: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    iat: usize,
    exp: usize,
}

impl AuthService {
    pub fn new(repo: PostgresRepo, jwt_secret: String, jwt_expiration: i64) -> Self {
        Self {
            repo,
            jwt_secret,
            jwt_expiration,
        }
    }

    pub async fn register(&self, name: String, email: String, password: String) -> Result<User> {
        if self
            .repo
            .get_user(None, None, Some(&email), None)
            .await?
            .is_some()
        {
            return Err(Error::BadRequest("Unavailable.".to_string()));
        }

        let verification_token = Uuid::now_v7();
        let expires_at = Utc::now() + Duration::hours(24);

        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|_| Error::Unauthorized)?
            .to_string();

        self.repo
            .create_user(
                name,
                email,
                password_hash,
                verification_token.to_string(),
                Some(expires_at),
            )
            .await
    }

    pub async fn login(&self, email: &str, password: &str) -> Result<String> {
        let user = self
            .repo
            .get_user(None, None, Some(email), None)
            .await?
            .ok_or(Error::BadRequest(
                "User not found, create an account!".to_string(),
            ))?;

        if !user.verified {
            return Err(Error::BadRequest("Check your e-email!".to_string()));
        }

        let argon2 = Argon2::default();
        let parsed_hash =
            PasswordHash::new(&user.password).map_err(|_| Error::InternalServerError)?;
        argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .map_err(|_| Error::BadRequest("Invalid password!".to_string()))?;
        self.generate_token(user.id, 60 * 60)
    }

    fn generate_token(&self, user_id: Uuid, expires_in_seconds: i64) -> Result<String> {
        let now = Utc::now();
        let exp = (now + Duration::minutes(expires_in_seconds)).timestamp() as usize;
        let iat = now.timestamp() as usize;
        let claims = Claims {
            sub: user_id.to_string(),
            iat,
            exp,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(|_| Error::InternalServerError)
    }

    pub async fn verify_email(&self, token: String) -> Result<String> {
        let user = self.repo.get_user(None, None, None, Some(&token)).await?;

        let user = user.ok_or(Error::BadRequest("Invalid data".to_string()))?;

        if let Some(expires_at) = user.token_expires_at {
            if expires_at < Utc::now() {
                return Err(Error::BadRequest("Token expired".to_string()));
            }
        }
        self.repo.verifed_token(&token).await?;
        send_welcome_email(&user.email, &user.name).await?;

        self.generate_token(user.id, self.jwt_expiration)
    }

    pub async fn forgot_password(&self, email: String) -> Result<()> {
        let user = self.repo.get_user(None, None, Some(&email), None).await?;

        let user = user.ok_or(Error::BadRequest("E-mail inválido!".to_string()))?;

        let verification_token = Uuid::now_v7().to_string();
        let expires_at = Utc::now() + Duration::minutes(30);

        let user_id =
            Uuid::parse_str(&user.id.to_string()).map_err(|_| Error::InternalServerError)?;

        self.repo
            .add_verifed_token(user_id, expires_at, &verification_token)
            .await?;

        let reset_link = format!(
            "{}/confirm-auth/reset-password?token={}",
            env::var("API_URL").expect("API_URL must be set"),
            &verification_token
        );

        let email_sent = send_forgot_password_email(&user.email, &reset_link, &user.name).await;

        if email_sent.is_err() {
            return Err(Error::InternalServerError);
        }

        Ok(())
    }

    pub async fn reset_password(&self, token: String, new_password: String) -> Result<()> {
        let user = self.repo.get_user(None, None, None, Some(&token)).await?;

        let user = user.ok_or(Error::Unauthorized)?;

        if let Some(expires_at) = user.token_expires_at {
            if expires_at < Utc::now() {
                return Err(Error::BadRequest("Token expired".to_string()));
            }
        }

        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(new_password.as_bytes(), &salt)
            .map_err(|_| Error::InternalServerError)?
            .to_string();

        let user_id =
            Uuid::parse_str(&user.id.to_string()).map_err(|_| Error::InternalServerError)?;

        self.repo.update_password(user_id, &password_hash).await?;

        self.repo.verifed_token(&token).await?;

        Ok(())
    }
}
