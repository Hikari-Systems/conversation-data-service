use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub id: Uuid,
    pub thread_id: Uuid,
    pub sender_id: String,
    pub content: String,
    // `tool_calls` is a nullable JSONB column. `serde_json::Value` decodes
    // JSON/JSONB natively (json feature) and `Option` handles SQL NULL — the
    // `#[sqlx(json)]` attribute here decoded it as non-nullable `Json<Value>`,
    // which failed on NULL rows with "unexpected null; try decoding as Option".
    pub tool_calls: Option<Value>,
    pub tool_result_call_id: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMessage {
    pub thread_id: Uuid,
    pub sender_id: String,
    pub content: String,
    pub tool_calls: Option<Value>,
    pub tool_result_call_id: Option<String>,
}

pub async fn insert(pool: &PgPool, input: CreateMessage) -> Result<Message> {
    let now = Utc::now();
    let msg = sqlx::query_as::<_, Message>(
        "INSERT INTO message
            (id, thread_id, sender_id, content, tool_calls, tool_result_call_id, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $7)
         RETURNING *",
    )
    .bind(Uuid::new_v4())
    .bind(input.thread_id)
    .bind(&input.sender_id)
    .bind(&input.content)
    .bind(input.tool_calls.as_ref().map(sqlx::types::Json))
    .bind(&input.tool_result_call_id)
    .bind(now)
    .fetch_one(pool)
    .await?;
    Ok(msg)
}

#[allow(dead_code)]
pub async fn get(pool: &PgPool, id: Uuid) -> Result<Option<Message>> {
    let msg = sqlx::query_as::<_, Message>("SELECT * FROM message WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(msg)
}

#[allow(dead_code)]
pub async fn get_all(pool: &PgPool) -> Result<Vec<Message>> {
    let msgs = sqlx::query_as::<_, Message>(
        "SELECT * FROM message ORDER BY created_at ASC",
    )
    .fetch_all(pool)
    .await?;
    Ok(msgs)
}

pub async fn get_all_by_thread_id(pool: &PgPool, thread_id: Uuid) -> Result<Vec<Message>> {
    let msgs = sqlx::query_as::<_, Message>(
        "SELECT * FROM message WHERE thread_id = $1 ORDER BY created_at ASC",
    )
    .bind(thread_id)
    .fetch_all(pool)
    .await?;
    Ok(msgs)
}

pub async fn get_sender_ids_by_thread_id(pool: &PgPool, thread_id: Uuid) -> Result<Vec<String>> {
    let rows = sqlx::query_scalar::<_, String>(
        "SELECT DISTINCT sender_id FROM message WHERE thread_id = $1",
    )
    .bind(thread_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

#[allow(dead_code)]
pub async fn delete(pool: &PgPool, id: Uuid) -> Result<()> {
    sqlx::query("DELETE FROM message WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}
