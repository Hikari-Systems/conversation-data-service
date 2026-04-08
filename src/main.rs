use actix_web::{web, App, HttpResponse};
use anyhow::Result;
use sqlx::PgPool;
use tracing::info;

use hs_utils::db::build_pool;

mod config;
mod models;
mod routes;

pub struct AppState {
    pub pool: PgPool,
}

#[actix_web::main]
async fn main() -> Result<()> {
    hs_utils::healthcheck::check_subcommand(
        config::load().map(|c| c.server.port).unwrap_or(3000),
    );

    let cfg = config::load()?;

    hs_utils::logging::init(&cfg.log.level);

    info!("Starting conversation-data-service");

    let pool = build_pool(&cfg.db).await?;

    sqlx::migrate!("./migrations").run(&pool).await?;
    info!("Migrations applied");

    let state = web::Data::new(AppState { pool });
    let port = cfg.server.port;

    hs_utils::server::run(port, move || {
        // POST /api/message accepts large tool outputs
        let json_cfg = web::JsonConfig::default().limit(50 * 1024 * 1024);

        App::new()
            .app_data(state.clone())
            .app_data(json_cfg)
            .route("/healthcheck", web::get().to(|| async { "OK" }))
            .route("/", web::get().to(root_page))
            .configure(routes::configure)
    })
    .await
}

async fn root_page() -> HttpResponse {
    static HTML: &str = include_str!("../static/index.html");
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(HTML)
}
