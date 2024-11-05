use axum::body::Body;
use axum::http::{HeaderMap, Method, Request, StatusCode};
use axum::middleware::{from_fn_with_state, Next};
use axum::response::Response;
use axum::routing::{delete, get, post, put};
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
    pub api_key: String,
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
pub struct UpdatePostFields {
    pub title: String,
    pub description: String,
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
        post_name: &str,
        new_images: Vec<String>,
    ) -> Result<Post, sqlx::Error> {
        // Busca o post pelo nome
        let mut post: Post = sqlx::query_as(
            "SELECT id, name, title, description, images FROM posts WHERE name = $1",
        )
        .bind(post_name)
        .fetch_one(&self.pool)
        .await?;

        // Adiciona as novas imagens ao vetor existente
        post.images.extend(new_images);

        // Atualiza o banco de dados com as novas imagens
        sqlx::query("UPDATE posts SET images = $1 WHERE name = $2")
            .bind(&post.images)
            .bind(post_name)
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

    pub async fn update_post(
        &self,
        id: Uuid,
        new_title: String,
        new_description: String,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE posts SET title = $1, description = $2 WHERE id = $3")
            .bind(new_title) // O novo título
            .bind(new_description) // A nova descrição
            .bind(id) // O ID do post
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn delete_post_by_name(&self, post_name: &str) -> Result<(), sqlx::Error> {
        // Deleta o post pelo nome
        sqlx::query("DELETE FROM posts WHERE name = $1")
            .bind(post_name)
            .execute(&self.pool)
            .await?;

        Ok(())
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
    Path(post_name): Path<String>,
    Json(add_images_request): Json<AddImagesRequest>,
) -> impl IntoResponse {
    match state
        .repository
        .add_images(&post_name, add_images_request.images)
        .await
    {
        Ok(post) => Ok((axum::http::StatusCode::OK, Json(post))),
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
    println!("{}", post_name);
    match state.repository.find_post_by_name(&post_name).await {
        Ok(Some(post)) => Ok(Json(post.images)), // Retorna as imagens
        Ok(None) => Err((axum::http::StatusCode::NOT_FOUND, "Post not found")),
        Err(_) => Err((
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "Database error",
        )),
    }
}

async fn update_post_fields(
    State(state): State<Arc<AppState>>,
    Path(post_id): Path<Uuid>, // Usar o ID do post na URL
    Json(update_fields): Json<UpdatePostFields>,
) -> impl IntoResponse {
    // Inverte o conteúdo de title e description
    let new_title = update_fields.description.clone();
    let new_description = update_fields.title.clone();

    match state
        .repository
        .update_post(post_id, new_title, new_description)
        .await
    {
        Ok(_) => Ok((axum::http::StatusCode::OK, "Post updated successfully")),
        Err(_) => Err((
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to update post",
        )),
    }
}

async fn delete_post(
    State(state): State<Arc<AppState>>,
    Path(post_name): Path<String>,
) -> impl IntoResponse {
    match state.repository.delete_post_by_name(&post_name).await {
        Ok(_) => (StatusCode::NO_CONTENT).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete post").into_response(),
    }
}

async fn require_api_key(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Request<Body>,
    next: Next,
) -> Result<Response, Response> {
    match headers.get("x-api-key") {
        Some(header_value) if header_value == &state.api_key => Ok(next.run(req).await),
        Some(_) => Err((StatusCode::UNAUTHORIZED, "Invalid API Key").into_response()),
        None => Err((StatusCode::UNAUTHORIZED, "Missing API Key").into_response()),
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let port = var("PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(8000);

    let api_key = var("API_KEY").expect("API_KEY must be set");

    let db_url = var("POSTGRES_URL").expect("POSTGRES_URL must be set");

    let repo = PostgresRepository::connect(&db_url).await;

    let app_state = Arc::new(AppState {
        repository: repo,
        api_key,
    });

    let allowed_origins = ["https://nextlevelcodeblog.netlify.app".parse().unwrap()];

    let cors = CorsLayer::new()
        .allow_origin(allowed_origins)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(Any);

    // build our application with a single route
    let app = Router::new()
        .route("/api/posts", get(list_posts))
        .route("/api/posts", post(create_post))
        .route("/api/post/:id", get(get_post_by_id))
        .route("/api/posts/:name/images", post(add_images_to_post))
        .route("/api/post/:name/images", get(get_images_by_post_name)) // Nova rota
        .route("/api/posts/update/:id", put(update_post_fields))
        .route("/api/delete/:name", delete(delete_post))
        .nest_service("/api/assets", ServeDir::new("src/assets"))
        .with_state(app_state.clone())
        .layer(cors)
        .layer(from_fn_with_state(app_state, require_api_key));

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    axum_server::bind(addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
