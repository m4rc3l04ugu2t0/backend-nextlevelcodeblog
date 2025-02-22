#![allow(unused)]
use std::sync::Arc;

use axum::{
    http::{
        header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
        HeaderValue, Method, StatusCode,
    },
    routing::get_service,
    Extension, Router,
};
use config::Config;
use dotenv::dotenv;
use repositories::PostgresRepo;
use routes::create_routes;
use services::auth::AuthService;
use sqlx::{postgres::PgPoolOptions, PgPool};
use tower_http::{cors::CorsLayer, services::ServeDir};
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
    pub db_pool: PgPool,
    pub config: Config,
    pub auth_service: AuthService,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer().pretty())
        .init();

    dotenv().ok();

    let config = Config::init();


let pool = match PgPoolOptions::new()
.max_connections(10)
.connect(&config.database_url)
.await
{
Ok(pool) => {
    println!("✅ Connection to the database is successful!");
    pool// 👈 Desabilita o cache de prepared statements
}
Err(err) => {
    println!("🔥 Failed to connect to the database: {:?}", err);
    std::process::exit(1);
}
};

    let db_blog = PostgresRepo::new(pool.clone());

    let cors = CorsLayer::new()
        .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE])
        .allow_credentials(true)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE]);

    let app_state = AppState {
        db_pool: pool,
        config: config.clone(),
        auth_service: AuthService::new(db_blog, config.jwt_secret.clone(), config.jwt_maxage),
    };

    let app = create_routes(Arc::new(app_state)).layer(cors);

    let listener = tokio::net::TcpListener::bind("[::]:8080").await.unwrap();
    info!("{} - {:?}", "LISTENING", listener.local_addr());
    axum::serve(listener, app).await.unwrap();
}
