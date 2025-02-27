use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2, PasswordHash, PasswordVerifier,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    models::users::{NameUpdateDto, User, UserPasswordUpdateDto},
    repositories::{user_repo::UserRepository, PostgresRepo},
    Error, Result,
};

use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};

#[derive(Clone)]
pub struct UserService {
    repo: PostgresRepo,
    jwt_secret: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    iat: usize,
    exp: usize,
}
impl UserService {
    pub fn new(repo: PostgresRepo, jwt_secret: String) -> Self {
        Self { repo, jwt_secret }
    }

    pub async fn get_user(
        &self,
        user_id: Option<Uuid>,
        name: Option<&str>,
        email: Option<&str>,
        token: Option<&str>,
    ) -> Result<User> {
        let user = self.repo.get_user(user_id, name, email, token).await?;
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

        Uuid::parse_str(&decode.claims.sub).map_err(|_| Error::Unauthorized)
    }

    pub async fn delete_user(&self, user_id: &str) -> Result<()> {
        let user_id = Uuid::parse_str(user_id).unwrap();

        self.repo.delete_user(user_id).await?;
        Ok(())
    }

    pub async fn update_username(
        &self,
        user: &User,
        user_update: NameUpdateDto,
    ) -> Result<()> {
        let argon2 = Argon2::default();
        let parsed_hash =
            PasswordHash::new(&user.password).map_err(|_| Error::InternalServerError)?;
        argon2
            .verify_password(user_update.password.as_bytes(), &parsed_hash)
            .map_err(|_| Error::BadRequest("Invalid password!".to_string()))?;

         self
            .repo
            .update_username(user.id, &user_update.name)
            .await?;

        Ok(())
    }

    pub async fn update_user_password(
        &self,
        user: &User,
        user_update: UserPasswordUpdateDto,
    ) -> Result<()> {
        let user_id = uuid::Uuid::parse_str(&user.id.to_string()).unwrap();

        let result = self.repo.get_user(Some(user_id), None, None, None).await?;

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

        self.repo.update_password(user_id, &hash_password).await?;
        Ok(())
    }
}
