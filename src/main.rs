
use axum::middleware::from_fn_with_state;
use config::Config;
use handlers::auth::{configure_cors, require_api_key};
use repositories::PostgresRepo;
use routes::create_routes;
use services::{
    auth::AuthService, posts::NewsPostsService, user::UserService, video::VideosService,
};
use sqlx::{postgres::PgPoolOptions, PgPool};

use std::{env, sync::Arc};

pub use self::errors::{Error, Result};

mod config;
mod errors;
mod handlers;
mod mail;
mod middleware;
mod models;
mod repositories;
mod routes;
mod services;

#[derive(Clone)]
pub struct AppState {
    pub api_key: String,
    pub db_pool: PgPool,
    pub config: Config,
    pub auth_service: AuthService,
    pub news_post_service: NewsPostsService,
    pub videos_service: VideosService,
    pub users_service: UserService,
}

#[tokio::main]
async fn main() {
    let config = Config::init();

    let api_key = env::var("API_KEY").unwrap_or_else(|_| {
        panic!("🔒 API_KEY environment variable must be set and non-empty!");
    });

    if api_key.is_empty() {
        panic!("🔒 API_KEY cannot be empty!");
    }

    let pool = match PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await
    {
        Ok(pool) => {
            println!("✅ Connection to the database is successful!");
            pool
        }
        Err(err) => {
            println!("🔥 Failed to connect to the database: {:?}", err);
            std::process::exit(1);
        }
    };

    let db_blog = PostgresRepo::new(pool.clone());

    let app_state = AppState {
        api_key,
        db_pool: pool,
        config: config.clone(),
        auth_service: AuthService::new(
            db_blog.clone(),
            config.jwt_secret.clone(),
            config.jwt_maxage,
        ),
        news_post_service: NewsPostsService::new(db_blog.clone()),
        users_service: UserService::new(db_blog.clone(), config.jwt_secret.clone()),
        videos_service: VideosService::new(db_blog),
    };

    let app = create_routes(Arc::new(app_state.clone()))
        .layer(configure_cors())
        .layer(from_fn_with_state(app_state, require_api_key));

    let listener = tokio::net::TcpListener::bind(format!(
        "[::]:{}",
        env::var("PORT").unwrap_or_else(|_| "8080".to_string())
    ))
    .await
    .unwrap();
    axum::serve(listener, app).await.unwrap();
}
