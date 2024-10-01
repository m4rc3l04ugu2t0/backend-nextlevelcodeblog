use axum::http::Method;
use axum::routing::{get, post};
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::env::var;
use std::{net::SocketAddr, sync::Arc};
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use uuid::Uuid;

// Definir o estado da aplicação
pub struct AppState {
    pub repository: PostgresRepository,
}

// Estruturas para os dados
#[derive(Serialize, sqlx::FromRow)]
pub struct Post {
    pub id: Uuid,
    pub name: String,
    pub title: String,
    pub description: String,
    pub images: Vec<String>, // Um array de URLs de imagens
}

#[derive(Clone, Deserialize, sqlx::FromRow)]
pub struct NewPost {
    pub name: String,
    pub title: String,
    pub description: String,
    pub images: Vec<String>, // Um array de URLs de imagens
}

#[derive(Deserialize)]
pub struct AddImagesRequest {
    pub images: Vec<String>, // Um vetor de URLs das imagens
}

#[derive(Clone)]
pub struct PostgresRepository {
    pub pool: PgPool,
}

impl PostgresRepository {
    pub async fn connect(db_url: &str) -> Self {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(db_url)
            .await
            .unwrap();

        PostgresRepository { pool }
    }

    // Função para buscar um post pelo ID
    pub async fn find_post(&self, id: Uuid) -> Result<Option<Post>, sqlx::Error> {
        sqlx::query_as("SELECT id, name, title, description, images FROM posts WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
    }

    // Função para listar todos os posts
    pub async fn list_posts(&self) -> Result<Vec<Post>, sqlx::Error> {
        sqlx::query_as("SELECT id, name, title, description, images FROM posts")
            .fetch_all(&self.pool)
            .await
    }

    pub async fn create_post(&self, new_post: NewPost) -> Result<Post, sqlx::Error> {
        sqlx::query_as(
            "INSERT INTO posts (id, name, title, description, images) 
         VALUES ($1, $2, $3, $4, $5)
         RETURNING id, name, title, description, images",
        )
        .bind(Uuid::now_v7())
        .bind(new_post.name)
        .bind(new_post.description)
        .bind(new_post.title)
        .bind(new_post.images)
        .fetch_one(&self.pool)
        .await
    }

    // Função para adicionar imagens a um post
    pub async fn add_images(
        &self,
        post_id: Uuid,
        new_images: Vec<String>,
    ) -> Result<Post, sqlx::Error> {
        let mut post: Post =
            sqlx::query_as("SELECT id, name, title, description, images FROM posts WHERE id = $1")
                .bind(post_id)
                .fetch_one(&self.pool)
                .await?;

        // Adiciona as novas imagens ao vetor existente
        post.images.extend(new_images);

        // Atualiza o banco de dados
        sqlx::query("UPDATE posts SET images = $1 WHERE id = $2")
            .bind(&post.images)
            .bind(post_id)
            .execute(&self.pool)
            .await?;

        Ok(post)
    }

    pub async fn find_post_by_name(&self, name: &str) -> Result<Option<Post>, sqlx::Error> {
        sqlx::query_as("SELECT id, name, title, description, images FROM posts WHERE name = $1")
            .bind(name)
            .fetch_optional(&self.pool)
            .await
    }
}

// Handler para buscar um post pelo ID
async fn get_post_by_id(
    State(state): State<Arc<AppState>>,
    Path(post_id): Path<Uuid>,
) -> impl IntoResponse {
    match state.repository.find_post(post_id).await {
        Ok(Some(post)) => Ok(Json(post)),
        Ok(None) => Err((axum::http::StatusCode::NOT_FOUND, "Post not found")),
        Err(_) => Err((
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "Database error",
        )),
    }
}

// Handler para listar todos os posts
async fn list_posts(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match state.repository.list_posts().await {
        Ok(posts) => Ok(Json(posts)),
        Err(_) => Err((
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to fetch posts",
        )),
    }
}

async fn create_post(
    State(state): State<Arc<AppState>>,
    Json(new_post): Json<NewPost>,
) -> impl IntoResponse {
    match state.repository.create_post(new_post).await {
        Ok(post) => Ok((axum::http::StatusCode::CREATED, Json(post))),
        Err(_) => Err((
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to create post",
        )),
    }
}

async fn add_images_to_post(
    State(state): State<Arc<AppState>>,
    Path(post_id): Path<Uuid>,
    Json(request): Json<AddImagesRequest>,
) -> impl IntoResponse {
    match state.repository.add_images(post_id, request.images).await {
        Ok(post) => Ok(Json(post)), // Retorna o post atualizado
        Err(_) => Err((
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to add images",
        )),
    }
}

async fn get_images_by_post_name(
    State(state): State<Arc<AppState>>,
    Path(post_name): Path<String>,
) -> impl IntoResponse {
    // Busque o post pelo nome
    match state.repository.find_post_by_name(&post_name).await {
        Ok(Some(post)) => Ok(Json(post.images)), // Retorna as imagens
        Ok(None) => Err((axum::http::StatusCode::NOT_FOUND, "Post not found")),
        Err(_) => Err((
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "Database error",
        )),
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let port = var("PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(8000);

    let db_url = var("DATABASE_URL").expect("DATABASE_URL must be set");

    // Conectar ao banco de dados
    let repo = PostgresRepository::connect(&db_url).await;

    let app_state = Arc::new(AppState { repository: repo });

    // Configura o middleware CORS
    let cors = CorsLayer::new()
        .allow_origin(Any) // Permitir requisições de qualquer origem
        .allow_methods([Method::GET]) // Permitir apenas GET e POST
        .allow_headers(Any); // Permitir qualquer cabeçalho

    // build our application with a single route
    let app = Router::new()
        .route("/api/posts", get(list_posts))
        .route("/api/posts", post(create_post))
        .route("/api/post/:id", get(get_post_by_id))
        .route("/api/posts/:id/images", post(add_images_to_post))
        .route("/api/post/:id/images", get(get_images_by_post_name)) // Nova rota
        .nest_service("/api/assets", ServeDir::new("src/assets"))
        .layer(cors)
        .with_state(app_state); // Aplica o CORS como camada

    // run our app with hyper, listening globally on port 3000
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    axum_server::bind(addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
