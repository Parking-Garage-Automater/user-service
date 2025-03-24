mod helpers;

use axum::http::header::CONTENT_TYPE;
use axum::http::{HeaderValue, Method};
use axum::routing::{get, post, put};
use axum::{Router};
use dotenvy::dotenv;
use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use std::env;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing::{info, log, subscriber, Level};

const PORT: i32 = 8080;

pub type AppStateType = Arc<RwLock<AppState>>;

#[derive(Clone)]
pub struct AppState {
    conn: DatabaseConnection,
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in the environment");
    let mut opt = ConnectOptions::new(db_url);
    opt.sqlx_logging(false)
        .sqlx_logging_level(log::LevelFilter::Info);
    let conn = Database::connect(opt)
        .await
        .expect("Database connection failed");

    Migrator::up(&conn, None).await.unwrap();

    let app_state: AppStateType = Arc::new(RwLock::new(AppState { conn }));

    let subscriber = tracing_subscriber::fmt()
        .json()
        .with_max_level(Level::INFO)
        .with_current_span(false)
        .finish();
    subscriber::set_global_default(subscriber).expect("Tracing subscriber couldn't be loaded");

    let app = Router::new()
        .route(
            "/api/manage/health",
            get(|| async { r#"{"status": "UP"}"# }),
        )
        .route("/api/users/{licence}", get(helpers::users::get_user_by_licence))
        .route("/api/login", post(helpers::users::signin_user))
        .route("/api/register", post(helpers::users::create_user))
        .route("/api/users/{id}", put(helpers::users::update_user))
        .route("/api/verify", get(helpers::auth::authorize))
    
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        )
        .layer(
            CorsLayer::new()
                .allow_headers([CONTENT_TYPE])
                .allow_methods([Method::GET, Method::POST])
                .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap()), // .allow_origin("https://xxx.xxx.xxx".parse::<HeaderValue>().unwrap())
        )
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(&format!("0.0.0.0:{}", PORT))
        .await
        .unwrap();

    info!("listening on {}", PORT);
    axum::serve(listener, app).await.unwrap();
}
