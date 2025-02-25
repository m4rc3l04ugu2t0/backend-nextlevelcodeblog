use std::{env, sync::Arc};

use axum::{
    body::Body, extract::Request, http::{
        header::{ACCEPT, AUTHORIZATION, CONTENT_DISPOSITION, CONTENT_TYPE}, HeaderName,  Method, StatusCode
    }, middleware::{from_fn_with_state, Next}, response::Response,
};
use config::Config;
use dotenv::dotenv;
use repositories::PostgresRepo;
use routes::create_routes;
use services::auth::AuthService;
use sqlx::{postgres::PgPoolOptions, PgPool};
use tower_http::cors::CorsLayer;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

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
}
fn configure_cors() -> CorsLayer {
    let x_api_key = HeaderName::from_static("x-api-key");

    CorsLayer::new()
        .allow_origin([
            "https://nextlevelcode-blog.vercel.app"
                .parse()
                .expect("Invalid origin format"),
        ])
        .allow_methods(vec![
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers(vec![
            AUTHORIZATION,
            CONTENT_TYPE,
            ACCEPT,
            x_api_key,
        ])
        .allow_credentials(true)
        .expose_headers(vec![CONTENT_DISPOSITION])
        .max_age(std::time::Duration::from_secs(86400))
}
async fn require_api_key(
    req: Request<Body>,
    next: Next,
) -> std::result::Result<Response, StatusCode> {
    if req.method() == Method::OPTIONS {
        return Ok(next.run(req).await);
    }

    let headers = req.headers();
    let api_key_header = HeaderName::from_static("x-api-key");

    match headers.get(&api_key_header) {
        Some(api_key_value) => {
            let stored_key = env::var("API_KEY").unwrap_or_default();

            if api_key_value.to_str().unwrap_or("") == stored_key {
                Ok(next.run(req).await)
            } else {
                Err(StatusCode::UNAUTHORIZED)
            }
        }
        None => Err(StatusCode::UNAUTHORIZED),
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer().pretty())
        .init();

    dotenv().ok();

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
        auth_service: AuthService::new(db_blog, config.jwt_secret.clone(), config.jwt_maxage),
    };

    let app = create_routes(Arc::new(app_state.clone())).layer(configure_cors()).layer(from_fn_with_state(app_state, require_api_key));

    let listener = tokio::net::TcpListener::bind(format!(
        "[::]:{}",
        env::var("PORT").unwrap_or_else(|_| "8080".to_string())
    ))
    .await
    .unwrap();
    info!("{} - {:?}", "LISTENING", listener.local_addr());
    axum::serve(listener, app).await.unwrap();
}
