use actix_web::{web, App, HttpResponse, HttpServer};
use anyhow::Result;
use sqlx::PgPool;
use tracing::info;
use tracing_subscriber::EnvFilter;

mod config;
mod db;
mod models;
mod routes;

pub struct AppState {
    pub pool: PgPool,
}

#[actix_web::main]
async fn main() -> Result<()> {
    let cfg = config::load()?;

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(&cfg.log.level))
        .init();

    info!("Starting conversation-data-service");

    let pool = db::build_pool(&cfg.db).await?;

    sqlx::migrate!("./migrations").run(&pool).await?;
    info!("Migrations applied");

    let state = web::Data::new(AppState { pool });
    let port = cfg.server.port;

    info!("Listening on port {}", port);

    HttpServer::new(move || {
        // POST /api/message accepts large tool outputs
        let json_cfg = web::JsonConfig::default().limit(50 * 1024 * 1024);

        App::new()
            .app_data(state.clone())
            .app_data(json_cfg)
            .route("/healthcheck", web::get().to(|| async { "OK" }))
            .route("/", web::get().to(root_page))
            .configure(routes::configure)
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await?;

    Ok(())
}

async fn root_page() -> HttpResponse {
    static HTML: &str = include_str!("../static/index.html");
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(HTML)
}
