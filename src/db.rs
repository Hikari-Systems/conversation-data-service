use anyhow::{Context, Result};
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions, PgSslMode},
    PgPool,
};

use crate::config::DbConfig;

pub async fn build_pool(cfg: &DbConfig) -> Result<PgPool> {
    let port: u16 = cfg.port.parse().context("db.port must be a number")?;

    let ssl_mode = match cfg.ssl.as_ref().and_then(|s| s.enabled) {
        Some(true) => {
            let verify = cfg.ssl.as_ref().and_then(|s| s.verify).unwrap_or(true);
            if verify {
                PgSslMode::VerifyFull
            } else {
                PgSslMode::Require
            }
        }
        _ => PgSslMode::Prefer,
    };

    let mut opts = PgConnectOptions::new()
        .host(&cfg.host)
        .port(port)
        .database(&cfg.database)
        .username(&cfg.username)
        .password(&cfg.password)
        .ssl_mode(ssl_mode);

    if let Some(ca_path) = cfg.ssl.as_ref().and_then(|s| s.ca_cert_file.as_ref()) {
        opts = opts.ssl_root_cert(ca_path);
    }

    let min = cfg.minpool.unwrap_or(0);
    let max = cfg.maxpool.unwrap_or(10);

    PgPoolOptions::new()
        .min_connections(min)
        .max_connections(max)
        .connect_with(opts)
        .await
        .context("Failed to connect to database")
}
