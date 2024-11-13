use axum::body::Body;
use axum::http::header::{
    ACCEPT, ACCESS_CONTROL_ALLOW_CREDENTIALS, ACCESS_CONTROL_ALLOW_HEADERS,
    ACCESS_CONTROL_ALLOW_ORIGIN, AUTHORIZATION, CONTENT_TYPE,
};
use axum::http::{HeaderValue, Method, Request, StatusCode};
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
use tower_http::cors::CorsLayer;
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
    pub cover_image: String,
}

#[derive(Clone, Deserialize, sqlx::FromRow)]
pub struct NewPost {
    pub name: String,
    pub title: String,
    pub description: String,
    pub cover_image: String,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct Video {
    pub id: Uuid,
    pub post_id: Uuid,
    pub url: String,
}

#[derive(Deserialize)]
pub struct NewVideoRequest {
    pub url: String,
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
    pub async fn find_post(&self, post_name: &str) -> Result<Option<Post>, sqlx::Error> {
        sqlx::query_as(
            "SELECT id, name, title, description, cover_image FROM posts WHERE name = $1",
        )
        .bind(post_name)
        .fetch_optional(&self.pool)
        .await
    }

    // Função para listar todos os posts
    pub async fn list_posts(&self) -> Result<Vec<Post>, sqlx::Error> {
        sqlx::query_as("SELECT id, name, title, description, cover_image FROM posts")
            .fetch_all(&self.pool)
            .await
    }

    pub async fn create_post(&self, new_post: NewPost) -> Result<Post, sqlx::Error> {
        sqlx::query_as(
            "INSERT INTO posts (id, name, title, description, cover_image)
         VALUES ($1, $2, $3, $4, $5)
         RETURNING id, name, title, description, cover_image",
        )
        .bind(Uuid::now_v7())
        .bind(new_post.name)
        .bind(new_post.title)
        .bind(new_post.description)
        .bind(new_post.cover_image)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn add_video(&self, post_id: Uuid, video_url: String) -> Result<Video, sqlx::Error> {
        sqlx::query_as(
            "INSERT INTO videos (id, post_id, url)
             VALUES ($1, $2, $3)
             RETURNING id, post_id, url",
        )
        .bind(Uuid::now_v7())
        .bind(post_id)
        .bind(video_url)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn list_videos(&self, post_id: Uuid) -> Result<Vec<Video>, sqlx::Error> {
        sqlx::query_as("SELECT id, post_id, url FROM videos WHERE post_id = $1")
            .bind(post_id)
            .fetch_all(&self.pool)
            .await
    }

    pub async fn add_videos(&self, post_id: Uuid, videos: Vec<String>) -> Result<(), sqlx::Error> {
        for video in videos {
            sqlx::query("INSERT INTO videos (post_id, url) VALUES ($1, $2)")
                .bind(post_id)
                .bind(video)
                .execute(&self.pool)
                .await?;
        }
        Ok(())
    }

    pub async fn get_videos_by_post_id(&self, post_id: Uuid) -> Result<Vec<String>, sqlx::Error> {
        let videos = sqlx::query_scalar("SELECT url FROM videos WHERE post_id = $1")
            .bind(post_id)
            .fetch_all(&self.pool)
            .await?;
        Ok(videos)
    }

    // Função para adicionar imagens a um post
    pub async fn add_images(&self, post_id: Uuid, images: Vec<String>) -> Result<(), sqlx::Error> {
        for image in images {
            sqlx::query("INSERT INTO images (post_id, url) VALUES ($1, $2)")
                .bind(post_id)
                .bind(image)
                .execute(&self.pool)
                .await?;
        }
        Ok(())
    }

    pub async fn get_images_by_post_id(&self, post_id: Uuid) -> Result<Vec<String>, sqlx::Error> {
        let images = sqlx::query_scalar("SELECT url FROM images WHERE post_id = $1")
            .bind(post_id)
            .fetch_all(&self.pool)
            .await?;
        Ok(images)
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
async fn get_post_by_name(
    State(state): State<Arc<AppState>>,
    Path(post_name): Path<String>,
) -> impl IntoResponse {
    match state.repository.find_post(&post_name).await {
        Ok(Some(post)) => Ok((StatusCode::OK, Json(post))),
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
        Ok(posts) => Ok((StatusCode::OK, Json(posts))),
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
        Ok(post) => Ok((StatusCode::CREATED, Json(post))),
        Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to create post")),
    }
}

async fn add_images_to_post(
    State(state): State<Arc<AppState>>,
    Path(post_id): Path<Uuid>,
    Json(images): Json<Vec<String>>,
) -> impl IntoResponse {
    match state.repository.add_images(post_id, images).await {
        Ok(_) => (StatusCode::OK, "Images added successfully").into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to add images").into_response(),
    }
}

async fn get_images_by_post_id(
    State(state): State<Arc<AppState>>,
    Path(post_id): Path<Uuid>,
) -> impl IntoResponse {
    match state.repository.get_images_by_post_id(post_id).await {
        Ok(images) => Ok((StatusCode::OK, Json(images))),
        Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch images")),
    }
}

async fn add_videos_to_post(
    State(state): State<Arc<AppState>>,
    Path(post_id): Path<Uuid>,
    Json(videos): Json<Vec<String>>,
) -> impl IntoResponse {
    match state.repository.add_videos(post_id, videos).await {
        Ok(_) => (StatusCode::OK, "Videos added successfully").into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to add videos").into_response(),
    }
}

async fn get_videos_by_post_id(
    State(state): State<Arc<AppState>>,
    Path(post_id): Path<Uuid>,
) -> impl IntoResponse {
    match state.repository.get_videos_by_post_id(post_id).await {
        Ok(videos) => Ok((StatusCode::OK, Json(videos))),
        Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch videos")),
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
        Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to update post")),
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

fn routes_static() -> Router {
    Router::new().nest_service("/api/assets", ServeDir::new("src/assets"))
}

async fn require_api_key(req: Request<Body>, next: Next) -> Result<Response, StatusCode> {
    // Se o método for OPTIONS, pule a autenticação
    if req.method() == axum::http::Method::OPTIONS {
        return Ok(next.run(req).await);
    }

    // Caso contrário, continue verificando a API_KEY normalmente
    let headers = req.headers();
    if let Some(api_key) = headers.get("X-Api-Key") {
        if api_key == "nextlevelcode" {
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

    let allowed_origins = ["https://nextlevelcodeblog.netlify.app"
        .parse::<HeaderValue>()
        .unwrap()];

    let cors = CorsLayer::new()
        .allow_origin(allowed_origins)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            CONTENT_TYPE,
            AUTHORIZATION,
            ACCEPT,
            ACCESS_CONTROL_ALLOW_ORIGIN,
            ACCESS_CONTROL_ALLOW_HEADERS,
            ACCESS_CONTROL_ALLOW_CREDENTIALS,
            "X-Api-Key".parse().unwrap(),
        ])
        .allow_credentials(true);

    // build our application with a single route
    let app = Router::new()
        .route("/api/posts", get(list_posts).post(create_post)) // Rota para listar e criar posts
        .route("/api/post/:name", get(get_post_by_name))
        .route("/api/posts/:post_id/images", post(add_images_to_post)) // Adicionar imagens a um post
        .route("/api/post/:name/images", get(get_images_by_post_id)) // Nova rota
        .route("/api/post/update/:id", put(update_post_fields))
        .route("/api/delete/:name", delete(delete_post))
        .route("/api/post/:post_id/videos", post(add_videos_to_post)) // Adicionar vídeos a um post
        .route("/api/post/:post_id/videos", get(get_videos_by_post_id)) // Obter vídeos de um post
        .with_state(app_state.clone())
        .layer(cors)
        .layer(from_fn_with_state(app_state, require_api_key))
        .fallback_service(routes_static());

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    axum_server::bind(addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
