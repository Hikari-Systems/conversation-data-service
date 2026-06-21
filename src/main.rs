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

#[derive(serde::Deserialize)]
struct HealthQuery {
    deps: Option<String>,
}

/// `GET /healthcheck` → `OK` (cheap liveness probe the Docker HEALTHCHECK
/// subcommand hits). `?deps=true` additionally runs `SELECT 1` against the
/// database, returning 503 when the pool is unhealthy.
async fn healthcheck(
    state: actix_web::web::Data<AppState>,
    q: actix_web::web::Query<HealthQuery>,
) -> actix_web::HttpResponse {
    let want_deps = matches!(q.deps.as_deref(), Some("true") | Some("1"));
    if !want_deps {
        return actix_web::HttpResponse::Ok().body("OK");
    }
    match sqlx::query("SELECT 1")
        .fetch_one(&state.pool)
        .await
    {
        Ok(_) => actix_web::HttpResponse::Ok().body("OK"),
        Err(e) => {
            tracing::error!("healthcheck db query failed: {e}");
            actix_web::HttpResponse::ServiceUnavailable().body("db down")
        }
    }
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
            .route("/healthcheck", web::get().to(healthcheck))
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
