#![allow(unused)]
use std::{env, sync::Arc};

use axum::{
    body::Body, extract::Request, http::{
        header::{ACCEPT, AUTHORIZATION, CONTENT_DISPOSITION, CONTENT_TYPE}, HeaderName, HeaderValue, Method, StatusCode
    }, middleware::{from_fn_with_state, Next}, response::Response, routing::get_service, Extension, Router
};
use config::Config;
use dotenv::dotenv;
use repositories::PostgresRepo;
use routes::create_routes;
use services::auth::AuthService;
use sqlx::{postgres::PgPoolOptions, PgPool};
use tower_http::{cors::CorsLayer, services::ServeDir, trace::TraceLayer};
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
    CorsLayer::new()
        .allow_origin([
            "https://nextlevelcode-blog.vercel.app".parse().unwrap(),
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
            HeaderName::from_static("X-Api-Key"),
        ])
        .allow_credentials(true)
        .expose_headers(vec![CONTENT_DISPOSITION])
        .max_age(std::time::Duration::from_secs(86400)) // 24 horas
}
async fn require_api_key(req: Request<Body>, next: Next) -> std::result::Result<Response, StatusCode> {
    // Se o método for OPTIONS, pule a autenticação
    if req.method() == axum::http::Method::OPTIONS {
        return Ok(next.run(req).await);
    }

    // Caso contrário, continue verificando a API_KEY normalmente
    let headers = req.headers();
    if let Some(api_key) = headers.get("X-Api-Key") {
        if api_key == env::var("API_KEY").unwrap().as_str() {
            Ok(next.run(req).await)
        } else {
            Err(StatusCode::UNAUTHORIZED)
        }
    } else {
        Err(StatusCode::UNAUTHORIZED)
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

    let api_key = env::var("API_KEY").expect("API_KEY must be set");

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

    let app = create_routes(Arc::new(app_state.clone())).layer(configure_cors()).layer(TraceLayer::new_for_http()).layer(from_fn_with_state(app_state, require_api_key));

    let listener = tokio::net::TcpListener::bind(format!(
        "[::]:{}",
        env::var("PORT").unwrap_or_else(|_| "8080".to_string())
    ))
    .await
    .unwrap();
    info!("{} - {:?}", "LISTENING", listener.local_addr());
    axum::serve(listener, app).await.unwrap();
}
