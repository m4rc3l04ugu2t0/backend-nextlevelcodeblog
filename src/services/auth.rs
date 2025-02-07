use std::sync::Arc;

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2, PasswordHash, PasswordVerifier,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    models::{
        posts::{CreatePostDto, Post},
        users::User,
    },
    repositories::{posts_repo::PostsRepository, user_repo::UserRepository, PostgresRepo},
    Error, Result,
};

#[derive(Clone)]
pub struct AuthService {
    user_repo: PostgresRepo,
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
    pub fn new(user_repo: PostgresRepo, jwt_secret: String, jwt_expiration: i64) -> Self {
        Self {
            user_repo,
            jwt_secret,
            jwt_expiration,
        }
    }

    pub async fn register(&self, name: String, email: String, password: String) -> Result<User> {
        if self.user_repo.find_by_email(&email).await?.is_some() {
            return Err(Error::BadRequest("Email already exists".to_string()));
        }

        let verification_token = Uuid::now_v7();
        let expires_at = Utc::now() + Duration::hours(24);

        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|_| Error::Unauthorized)?
            .to_string();

        self.user_repo
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
            .user_repo
            .find_by_email(email)
            .await?
            .ok_or(Error::Unauthorized)?;

        let argon2 = Argon2::default();
        let parsed_hash =
            PasswordHash::new(&user.password).map_err(|_| Error::InternalServerError)?;
        argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .map_err(|_| Error::Unauthorized)?;
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
        let user = self
            .user_repo
            .get_user(None, None, None, Some(&token))
            .await?;

        let user = user.ok_or(Error::NotFound)?;

        if let Some(expires_at) = user.token_expires_at {
            if expires_at < Utc::now() {
                return Err(Error::BadRequest("Token expired".to_string()));
            }
        }
        self.user_repo.verifed_token(token).await?;

        self.generate_token(user.id, self.jwt_expiration)
    }

    pub async fn forgot_password(&self, email: String) -> Result<()> {
        let user = self
            .user_repo
            .get_user(None, None, Some(&email), None)
            .await?;

        let user = user.ok_or(Error::NotFound)?;

        let verfication_token = Uuid::now_v7().to_string();
        let expires_at = Utc::now() + Duration::minutes(30);

        let user_id =
            Uuid::parse_str(&user.id.to_string()).map_err(|_| Error::InternalServerError)?;

        self.user_repo
            .add_verifed_token(user_id, expires_at, &verfication_token)
            .await?;

        Ok(())
    }

    pub async fn reset_password(&self, token: String, new_password: String) -> Result<()> {
        let user = self
            .user_repo
            .get_user(None, None, None, Some(&token))
            .await?;

        let user = user.ok_or(Error::NotFound)?;

        if let Some(expires_at) = user.token_expires_at {
            if expires_at < Utc::now() {
                return Err(Error::BadRequest("Token expired".to_string()));
            }
        }

        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(new_password.as_bytes(), &salt)
            .map_err(|_| Error::Unauthorized)?
            .to_string();

        let user_id =
            Uuid::parse_str(&user.id.to_string()).map_err(|_| Error::InternalServerError)?;

        self.user_repo
            .update_password(user_id, &password_hash)
            .await?;

        self.user_repo.verifed_token(token).await?;

        Ok(())
    }

    pub async fn get_user(
        &self,
        user_id: Option<Uuid>,
        name: Option<&str>,
        email: Option<&str>,
        token: Option<&str>,
    ) -> Result<User> {
        let user = self.user_repo.get_user(user_id, name, email, token).await?;
        let user = user.ok_or(Error::NotFound)?;
        Ok(user)
    }

    pub fn decode_token<T: Into<String>>(&self, token: T) -> Result<Uuid> {
        let decode = decode::<Claims>(
            &token.into(),
            &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &Validation::new(Algorithm::HS256),
        )
        .map_err(|_| Error::NotFound)?;

        Ok(Uuid::parse_str(&decode.claims.sub).map_err(|_| Error::Unauthorized)?)
    }

    pub async fn get_posts(&self) -> Result<Vec<Post>> {
        let posts = self.user_repo.get_posts().await?;
        Ok(posts)
    }

    pub async fn create_post(
        &self,
        user_id: &str,
        title: &str,
        description: &str,
        cover_image: &str,
    ) -> Result<Post> {
        let new_post = self
            .user_repo
            .create_post(user_id, title, description, cover_image)
            .await?;

        Ok(new_post)
    }

    pub async fn update_post(
        &self,
        user_id: &str,
        title: &str,
        description: &str,
        cover_image: &str,
    ) -> Result<Post> {
        let updated_post = self
            .user_repo
            .update_post(
                &user_id,
                Some(&title),
                Some(&description),
                Some(&cover_image),
            )
            .await?;

        Ok(updated_post)
    }

    pub async fn delete_post(&self, post_id: &str) -> Result<()> {
        let deleted_post = self.user_repo.delete_post(&post_id).await?;

        Ok(())
    }
}
