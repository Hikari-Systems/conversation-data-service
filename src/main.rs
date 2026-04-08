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

/// Makes a raw HTTP/1.1 GET to /healthcheck using only stdlib TCP.
/// Returns true on HTTP 200, false on any error or non-200 response.
/// Invoked when the binary is called as: server healthcheck [hostname [port]]
fn run_healthcheck(host: &str, port: u16) -> bool {
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use std::time::Duration;

    let Ok(mut stream) = TcpStream::connect(format!("{host}:{port}")) else {
        return false;
    };
    stream.set_read_timeout(Some(Duration::from_secs(4))).ok();
    stream.set_write_timeout(Some(Duration::from_secs(4))).ok();

    let req = "GET /healthcheck HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n";
    if stream.write_all(req.as_bytes()).is_err() {
        return false;
    }

    let mut response = String::new();
    if stream.read_to_string(&mut response).is_err() {
        return false;
    }

    response.starts_with("HTTP/1.1 200")
}

#[actix_web::main]
async fn main() -> Result<()> {
    if std::env::args().nth(1).as_deref() == Some("healthcheck") {
        let host = std::env::args().nth(2).unwrap_or_else(|| "localhost".to_string());
        let default_port = config::load().map(|c| c.server.port).unwrap_or(3000);
        let port = std::env::args().nth(3)
            .and_then(|s| s.parse::<u16>().ok())
            .unwrap_or(default_port);
        std::process::exit(if run_healthcheck(&host, port) { 0 } else { 1 });
    }

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
