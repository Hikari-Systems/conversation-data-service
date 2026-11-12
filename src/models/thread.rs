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
    /// The application that wrote the thread, as its caller named it. NULL when
    /// the caller didn't say (and for rows that predate the column).
    pub app: Option<String>,
    /// What sort of conversation it is within that application, as its caller
    /// named it. NULL when the caller didn't say.
    pub kind: Option<String>,
}

/// Every column `Thread` reads, in place of `SELECT *` / `RETURNING *` (a `*` statement's
/// cached plan breaks on old instances mid-deploy when a migration changes the table).
/// When a migration adds a column to `thread`, add it here and to `Thread` in the same change.
pub const THREAD_COLUMNS: &str =
    "id, title, visible_to_user_ids, bot_id, created_at, updated_at, app, kind";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateThread {
    pub title: String,
    pub visible_to_user_ids: Option<Vec<String>>,
    pub bot_id: Option<String>,
    /// Optional on the wire so callers that predate the columns keep working
    /// (their rows are stored with NULL `app` / `kind`).
    #[serde(default)]
    pub app: Option<String>,
    #[serde(default)]
    pub kind: Option<String>,
}

/// Optional exact-match filters for `GET /api/thread/byUserId/{userId}`.
/// An absent filter does not constrain the list (the behaviour before the
/// columns existed); a present one matches that value exactly, so it never
/// matches a NULL-tagged row.
#[derive(Debug, Default, Deserialize)]
pub struct ThreadFilter {
    pub app: Option<String>,
    pub kind: Option<String>,
}

#[tracing::instrument(skip_all, name = "db.thread.insert", fields(db.system.name = "postgresql"))]
pub async fn insert(pool: &PgPool, input: CreateThread) -> Result<Thread> {
    let now = Utc::now();
    // Not RETURNING *: THREAD_COLUMNS must stay in sync with thread's full column list (cached-plan breakage mid-deploy).
    let thread = sqlx::query_as::<_, Thread>(&format!(
        "INSERT INTO thread (id, title, visible_to_user_ids, bot_id, created_at, updated_at, app, kind)
         VALUES ($1, $2, $3, $4, $5, $5, $6, $7)
         RETURNING {THREAD_COLUMNS}"
    ))
    .bind(Uuid::new_v4())
    .bind(&input.title)
    .bind(&input.visible_to_user_ids)
    .bind(&input.bot_id)
    .bind(now)
    .bind(&input.app)
    .bind(&input.kind)
    .fetch_one(pool)
    .await?;
    Ok(thread)
}

#[tracing::instrument(skip_all, name = "db.thread.get", fields(db.system.name = "postgresql"))]
pub async fn get(pool: &PgPool, id: Uuid) -> Result<Option<Thread>> {
    // Not SELECT *: THREAD_COLUMNS must stay in sync with thread's full column list (cached-plan breakage mid-deploy).
    let thread = sqlx::query_as::<_, Thread>(&format!(
        "SELECT {THREAD_COLUMNS} FROM thread WHERE id = $1"
    ))
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(thread)
}

#[allow(dead_code)]
#[tracing::instrument(skip_all, name = "db.thread.get_all", fields(db.system.name = "postgresql"))]
pub async fn get_all(pool: &PgPool) -> Result<Vec<Thread>> {
    // Not SELECT *: THREAD_COLUMNS must stay in sync with thread's full column list (cached-plan breakage mid-deploy).
    let threads = sqlx::query_as::<_, Thread>(&format!(
        "SELECT {THREAD_COLUMNS} FROM thread ORDER BY created_at DESC"
    ))
    .fetch_all(pool)
    .await?;
    Ok(threads)
}

#[tracing::instrument(skip_all, name = "db.thread.get_all_by_user_id", fields(db.system.name = "postgresql"))]
pub async fn get_all_by_user_id(
    pool: &PgPool,
    user_id: &str,
    filter: &ThreadFilter,
) -> Result<Vec<Thread>> {
    // Not SELECT *: THREAD_COLUMNS must stay in sync with thread's full column list (cached-plan breakage mid-deploy).
    // The visibility test is parenthesised so the app/kind filters apply to
    // public (NULL-visibility) rows too.
    let threads = sqlx::query_as::<_, Thread>(&format!(
        "SELECT {THREAD_COLUMNS} FROM thread
         WHERE (visible_to_user_ids IS NULL OR $1 = ANY(visible_to_user_ids))
           AND ($2::text IS NULL OR app = $2)
           AND ($3::text IS NULL OR kind = $3)
         ORDER BY created_at DESC"
    ))
    .bind(user_id)
    .bind(&filter.app)
    .bind(&filter.kind)
    .fetch_all(pool)
    .await?;
    Ok(threads)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_body_without_app_or_kind_still_parses() {
        // Callers that predate the columns must keep working, so this service
        // and its callers can deploy in any order.
        let c: CreateThread =
            serde_json::from_str(r#"{"title":"t","visibleToUserIds":["u"],"botId":null}"#).unwrap();
        assert_eq!(c.app, None);
        assert_eq!(c.kind, None);
    }

    #[test]
    fn create_body_carries_app_and_kind() {
        let c: CreateThread = serde_json::from_str(
            r#"{"title":"t","visibleToUserIds":["u"],"app":"app-a","kind":"kind-x"}"#,
        )
        .unwrap();
        assert_eq!(c.app.as_deref(), Some("app-a"));
        assert_eq!(c.kind.as_deref(), Some("kind-x"));
    }

    #[test]
    fn thread_serialises_app_and_kind_even_when_null() {
        // Readers tell "untagged row" (null) from "service predates the field"
        // (absent) by this; keep both keys present.
        let t = Thread {
            id: Uuid::nil(),
            title: "t".into(),
            visible_to_user_ids: Some(vec!["u".into()]),
            bot_id: None,
            created_at: None,
            updated_at: None,
            app: None,
            kind: Some("kind-y".into()),
        };
        let v = serde_json::to_value(&t).unwrap();
        assert_eq!(v["app"], serde_json::Value::Null);
        assert_eq!(v["kind"], "kind-y");
        assert!(v.as_object().unwrap().contains_key("app"));
    }

    #[test]
    fn thread_columns_name_every_struct_field() {
        // THREAD_COLUMNS and `Thread` must change together (no SELECT *).
        let t = Thread {
            id: Uuid::nil(),
            title: String::new(),
            visible_to_user_ids: None,
            bot_id: None,
            created_at: None,
            updated_at: None,
            app: None,
            kind: None,
        };
        let v = serde_json::to_value(&t).unwrap();
        let cols: Vec<&str> = THREAD_COLUMNS.split(',').map(str::trim).collect();
        assert_eq!(cols.len(), v.as_object().unwrap().len());
        assert!(cols.contains(&"app") && cols.contains(&"kind"));
    }

    #[test]
    fn filter_parses_from_the_query_string() {
        let none: ThreadFilter = serde_urlencoded_from("");
        assert!(none.app.is_none() && none.kind.is_none());
        let both: ThreadFilter = serde_urlencoded_from("app=app-b&kind=kind-y");
        assert_eq!(both.app.as_deref(), Some("app-b"));
        assert_eq!(both.kind.as_deref(), Some("kind-y"));
        // Unknown parameters are ignored, as before.
        let other: ThreadFilter = serde_urlencoded_from("app=app-c&page=2");
        assert_eq!(other.app.as_deref(), Some("app-c"));
        assert!(other.kind.is_none());
    }

    fn serde_urlencoded_from(q: &str) -> ThreadFilter {
        actix_web::web::Query::<ThreadFilter>::from_query(q)
            .unwrap()
            .into_inner()
    }
}
