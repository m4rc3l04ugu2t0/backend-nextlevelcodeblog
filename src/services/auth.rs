use std::{env, sync::Arc};

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2, PasswordHash, PasswordVerifier,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

use crate::{
    errors::ValidationResponse,
    mail::mails::{send_forgot_password_email, send_welcome_email},
    models::{
        news_post::{
            CreateNewsPostDto, NewsPost, PostComment, PostCommentWithAuthor,
            PostCommentWithComments, UpdateNewsPost,
        },
        query::{CreateCategory, CreateVideo, ReturnVideo, Video, VideoDto},
        response::Response,
        users::{FilterUserDto, NameUpdateDto, User, UserPasswordUpdateDto},
    },
    repositories::{news_post_repo::NewsPostsRepository, user_repo::UserRepository, PostgresRepo},
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
        if self
            .user_repo
            .get_user(None, None, Some(&email), None)
            .await?
            .is_some()
        {
            return Err(Error::BadRequest(
                "Email already exists.".to_string(),
            ));
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
        let user = self
            .user_repo
            .get_user(None, None, None, Some(&token))
            .await?;

        let user = user.ok_or(Error::BadRequest("Invalid data".to_string()))?;

        if let Some(expires_at) = user.token_expires_at {
            if expires_at < Utc::now() {
                return Err(Error::BadRequest("Token expired".to_string()));
            }
        }
        self.user_repo.verifed_token(&token).await?;
        // self.user_repo.verifed_token(&user.verification_token).await?;
        send_welcome_email(&user.email, &user.name).await?;

        self.generate_token(user.id, self.jwt_expiration)
    }

    pub async fn forgot_password(&self, email: String) -> Result<()> {
        let user = self
            .user_repo
            .get_user(None, None, Some(&email), None)
            .await?;

        let user = user.ok_or(Error::BadRequest("E-mail inválido!".to_string()))?;

        let verification_token = Uuid::now_v7().to_string();
        let expires_at = Utc::now() + Duration::minutes(30);

        let user_id =
            Uuid::parse_str(&user.id.to_string()).map_err(|_| Error::InternalServerError)?;

        self.user_repo
            .add_verifed_token(user_id, expires_at, &verification_token)
            .await?;

        let reset_link = format!(
            "{}/confirm-auth/reset-password?token={}", env::var("API_URL").expect("API_URL must be set"),
            &verification_token
        );

        let email_sent = send_forgot_password_email(&user.email, &reset_link, &user.name).await;

        if let Err(e) = email_sent {
            return Err(Error::InternalServerError);
        }

        Ok(())
    }

    pub async fn reset_password(&self, token: String, new_password: String) -> Result<()> {
        let user = self
            .user_repo
            .get_user(None, None, None, Some(&token))
            .await?;

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

        self.user_repo
            .update_password(user_id, &password_hash)
            .await?;

        self.user_repo.verifed_token(&token).await?;

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

    pub async fn delete_user(&self, user_id: &str) -> Result<()> {
        let user_id = Uuid::parse_str(user_id).unwrap();

        self.user_repo.delete_user(user_id).await?;
        Ok(())
    }

    pub async fn update_username(
        &self,
        user: &User,
        user_update: NameUpdateDto,
    ) -> Result<FilterUserDto> {
        let user_id = uuid::Uuid::parse_str(&user.id.to_string()).unwrap();

        let argon2 = Argon2::default();
        let parsed_hash =
            PasswordHash::new(&user.password).map_err(|_| Error::InternalServerError)?;
        argon2
            .verify_password(user_update.password.as_bytes(), &parsed_hash)
            .map_err(|_| Error::BadRequest("Invalid password!".to_string()))?;

        let result = self
            .user_repo
            .update_username(user.id, &user_update.name)
            .await?;

        let filtered_user = FilterUserDto::filter_user(&result);

        Ok(filtered_user)
    }

    pub async fn update_user_password(
        &self,
        user: &User,
        user_update: UserPasswordUpdateDto,
    ) -> Result<()> {
        let user_id = uuid::Uuid::parse_str(&user.id.to_string()).unwrap();

        let result = self
            .user_repo
            .get_user(Some(user_id), None, None, None)
            .await?;

        let user = result.ok_or(Error::BadRequest("Invalid password!".to_string()))?;

        let argon2 = Argon2::default();
        let parsed_hash =
            PasswordHash::new(&user.password).map_err(|_| Error::InternalServerError)?;
        argon2
            .verify_password(user_update.old_password.as_bytes(), &parsed_hash)
            .map_err(|_| Error::BadRequest("Invalid password!".to_string()))?;

        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let hash_password = argon2
            .hash_password(user_update.new_password.as_bytes(), &salt)
            .map_err(|_| Error::Unauthorized)?
            .to_string();

        self.user_repo
            .update_password(user_id, &hash_password)
            .await?;
        Ok(())
    }

    pub async fn get_news_posts(&self) -> Result<Vec<NewsPost>> {
        let newspost = self.user_repo.get_news_posts().await?;

        Ok(newspost)
    }

    pub async fn create_news_post(
        &self,
        news_post: CreateNewsPostDto,
        author_id: &str,
    ) -> Result<NewsPost> {
        let news_post = self
            .user_repo
            .create_news_post(
                &news_post.url,
                &news_post.description,
                &author_id,
                &news_post.author_name,
            )
            .await?;

        Ok(news_post)
    }

    pub async fn update_news_post(
        &self,
        news_post_id: &str,
        update_news_post_url: Option<&str>,
        update_news_post_description: Option<&str>,
    ) -> Result<NewsPost> {
        let update_news_post = self
            .user_repo
            .update_news_post(
                news_post_id,
                update_news_post_url,
                update_news_post_description,
            )
            .await?;

        Ok(update_news_post)
    }

    pub async fn delete_news_post(&self, news_post_id: &str) -> Result<()> {
        self.user_repo.delete_news_post(news_post_id).await?;
        Ok(())
    }

    pub async fn videos(&self) -> Result<Vec<Video>> {
        let videos = self.user_repo.videos().await?;
        Ok(videos)
    }

    pub async fn create_video(
        &self,
        title: &str,
        youtube_id: &str,
        duration: &str,
        views: Option<i32>,
    ) -> Result<()> {
        let id = Uuid::now_v7();
        self.user_repo
            .create_video(id, title, youtube_id, duration, views)
            .await?;
        Ok(())
    }

    pub async fn update_video(
        &self,
        video_id: &str,
        title: Option<&str>,
        youtube_id: Option<&str>,
        duration: Option<&str>,
        views: Option<i32>,
    ) -> Result<()> {
        let video_id = Uuid::parse_str(video_id).unwrap();
        self.user_repo
            .update_video(video_id, title, youtube_id, duration, views)
            .await?;

        Ok(())
    }

    pub async fn delete_video(&self, video_id: &str) -> Result<()> {
        let video_id = Uuid::parse_str(video_id).unwrap();
        self.user_repo.delete_video(video_id).await?;

        Ok(())
    }

    pub async fn create_category(&self, category: &str) -> Result<CreateCategory> {
        let category_id = Uuid::now_v7();
        let category = self
            .user_repo
            .create_category(category_id, category)
            .await?;

        Ok(category)
    }

    pub async fn delete_category(&self, category_id: &str) -> Result<()> {
        let category_id = Uuid::parse_str(category_id).unwrap();
        self.user_repo.delete_category(category_id).await?;
        Ok(())
    }

    pub async fn add_category_to_video(&self, video_id: &str, category_id: &str) -> Result<()> {
        let video_id = Uuid::parse_str(video_id).unwrap();
        let category_id = Uuid::parse_str(category_id).unwrap();
        self.user_repo
            .add_category_to_video(video_id, category_id)
            .await?;

        Ok(())
    }

    pub async fn remove_category_from_video(
        &self,
        video_id: &str,
        category_id: &str,
    ) -> Result<()> {
        let video_id = Uuid::parse_str(video_id).unwrap();
        let category_id = Uuid::parse_str(category_id).unwrap();
        self.user_repo
            .remove_category_from_video(video_id, category_id)
            .await?;
        Ok(())
    }

    pub async fn get_comments_for_post(&self, post_id: &str) -> Result<Vec<PostCommentWithAuthor>> {
        let comments = self.user_repo.get_comments_for_post(&post_id).await?;

        Ok(comments)
    }

    pub async fn get_video_by_youtube_id(&self, youtube_id: &str) -> Result<ReturnVideo> {
        let video = self.user_repo.get_video_by_youtube_id(&youtube_id).await?;

        Ok(video)
    }

    pub async fn create_comment(
        &self,
        post_id: &str,
        content: &str,
        author_id: &str,
        author_name: &str,
    ) -> Result<PostComment> {
        let created_comment = self
            .user_repo
            .create_comment(post_id, content, author_id, author_name)
            .await?;
        Ok(created_comment)
    }

    pub async fn update_comment(
        &self,
        comment_id: &str,
        content: Option<&str>,
    ) -> Result<PostComment> {
        let updated_comment = self.user_repo.update_comment(comment_id, content).await?;
        Ok(updated_comment)
    }

    pub async fn delete_comment(&self, comment_id: &str) -> Result<()> {
        self.user_repo.delete_comment(comment_id).await?;

        Ok(())
    }

    pub async fn get_posts_with_comments(&self, post_id: &str) -> Result<PostCommentWithComments> {
        let posts = self.user_repo.get_posts_with_comments(post_id).await?;

        Ok(posts)
    }

    pub async fn get_all_posts_with_comments(&self) -> Result<Vec<PostCommentWithComments>> {
        let posts_with_comments = self.user_repo.get_all_posts_with_comments().await?;

        Ok(posts_with_comments)
    }
}
