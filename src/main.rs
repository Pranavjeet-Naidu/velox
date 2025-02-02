use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use deadpool_redis::redis::{self, AsyncCommands};
use serde::{Deserialize, Serialize};
use rand::Rng;
use thiserror::Error;
use url::Url;

#[derive(Error, Debug)]
enum AppError {
    #[error("Redis error: {0}")]
    Redis(#[from] deadpool_redis::redis::RedisError),
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
    #[error("Pool error: {0}")]
    Pool(#[from] deadpool_redis::PoolError),
}

impl actix_web::error::ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        match self {
            AppError::Redis(_) => HttpResponse::InternalServerError().json("Database error"),
            AppError::InvalidUrl(msg) => HttpResponse::BadRequest().json(msg),
            AppError::Pool(_) => HttpResponse::ServiceUnavailable().json("Service temporarily unavailable"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct UrlMapping {
    original: String,
    shortened: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct ShortenedUrl {
    original: String,
    shortened: String,
}

#[derive(Deserialize)]
struct OriginalUrl {
    original: String,
}

struct AppState {
    pool: deadpool_redis::Pool,
    base_url: String,
}

fn generate_short_code() -> String {
    let mut rng = rand::thread_rng();
    let random_number: u64 = rng.gen();
    base62::encode(random_number).chars().take(8).collect()
}


async fn index() -> impl Responder {
    HttpResponse::Ok().json("Welcome to Velox URL Shortener!")
}

async fn shorten_url(
    data: web::Data<AppState>,
    url_req: web::Json<OriginalUrl>,
) -> Result<HttpResponse, AppError> {
    // Validate URL first
    validate_url(&url_req.original)?;
    
    let mut conn = data.pool.get().await?;
    let shortened = generate_short_code();

    redis::cmd("SET")
        .arg(&shortened)
        .arg(&url_req.original)
        .query_async::<_, ()>(&mut conn)
        .await?;

    let response = ShortenedUrl {
        original: url_req.original.clone(),
        shortened: format!("{}/{}", data.base_url, shortened),
    };

    tracing::info!("Created shortened URL: {:?}", response);
    Ok(HttpResponse::Ok().json(response))
}

fn validate_url(url: &str) -> Result<(), AppError> {
    match Url::parse(url) {
        Ok(_) => Ok(()),
        Err(_) => Err(AppError::InvalidUrl("Invalid URL format".to_string())),
    }
}
async fn get_original_url(
    data: web::Data<AppState>,
    shortened: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let mut conn = data.pool.get().await?;
    
    let original: Option<String> = conn.get(&shortened.into_inner()).await?;

    match original {
        Some(url) => Ok(HttpResponse::PermanentRedirect()
            .append_header(("Location", url))
            .finish()),
        None => Ok(HttpResponse::NotFound().json("URL not found")),
    }
}

async fn health_check(data: web::Data<AppState>) -> impl Responder {
    let result = data.pool.get().await;
    match result {
        Ok(_) => HttpResponse::Ok().json("Healthy"),
        Err(_) => HttpResponse::ServiceUnavailable().json("Unhealthy"),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Configure Redis connection pool
    let cfg = deadpool_redis::Config::from_url("redis://127.0.0.1/");
    let pool = cfg.create_pool(Some(deadpool_redis::Runtime::Tokio1)).unwrap();

    // Configure base URL for shortened URLs
    let base_url = std::env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:8082".to_string());
    
    let app_state = web::Data::new(AppState { pool, base_url });

    tracing::info!("Starting server at http://localhost:8082");

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/", web::get().to(index))
            .route("/shorten", web::post().to(shorten_url))
            .route("/health", web::get().to(health_check))
            .route("/{shortened}", web::get().to(get_original_url))
    })
    .bind("127.0.0.1:8082")?
    .run()
    .await
}