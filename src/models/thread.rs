use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Thread {
    pub id: Uuid,
    pub title: String,
    pub visible_to_user_ids: Option<Vec<String>>,
    pub bot_id: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateThread {
    pub title: String,
    pub visible_to_user_ids: Option<Vec<String>>,
    pub bot_id: Option<String>,
}

pub async fn insert(pool: &PgPool, input: CreateThread) -> Result<Thread> {
    let now = Utc::now();
    let thread = sqlx::query_as::<_, Thread>(
        "INSERT INTO thread (id, title, visible_to_user_ids, bot_id, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $5)
         RETURNING *",
    )
    .bind(Uuid::new_v4())
    .bind(&input.title)
    .bind(&input.visible_to_user_ids)
    .bind(&input.bot_id)
    .bind(now)
    .fetch_one(pool)
    .await?;
    Ok(thread)
}

pub async fn get(pool: &PgPool, id: Uuid) -> Result<Option<Thread>> {
    let thread = sqlx::query_as::<_, Thread>("SELECT * FROM thread WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(thread)
}

#[allow(dead_code)]
pub async fn get_all(pool: &PgPool) -> Result<Vec<Thread>> {
    let threads =
        sqlx::query_as::<_, Thread>("SELECT * FROM thread ORDER BY created_at DESC")
            .fetch_all(pool)
            .await?;
    Ok(threads)
}

pub async fn get_all_by_user_id(pool: &PgPool, user_id: &str) -> Result<Vec<Thread>> {
    let threads = sqlx::query_as::<_, Thread>(
        "SELECT * FROM thread
         WHERE visible_to_user_ids IS NULL OR $1 = ANY(visible_to_user_ids)
         ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(threads)
}
