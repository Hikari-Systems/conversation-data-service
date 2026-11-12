use actix_web::{web, HttpResponse};
use uuid::Uuid;

use crate::models::thread::{self, CreateThread, ThreadFilter};
use crate::AppState;

pub async fn get_thread(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> actix_web::Result<HttpResponse> {
    match thread::get(&state.pool, path.into_inner())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    {
        Some(t) => Ok(HttpResponse::Ok().json(t)),
        None => Ok(HttpResponse::NoContent().finish()),
    }
}

pub async fn get_threads_by_user_id(
    state: web::Data<AppState>,
    path: web::Path<String>,
    filter: web::Query<ThreadFilter>,
) -> actix_web::Result<HttpResponse> {
    let threads = thread::get_all_by_user_id(&state.pool, &path.into_inner(), &filter)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(threads))
}

pub async fn create_thread(
    state: web::Data<AppState>,
    body: web::Json<CreateThread>,
) -> actix_web::Result<HttpResponse> {
    let t = thread::insert(&state.pool, body.into_inner())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;
    Ok(HttpResponse::Created().json(t))
}

/// Route tests against a real Postgres. `#[ignore]`d because `cargo test` has no
/// database in this estate; run them on demand against a throwaway one:
///
/// ```sh
/// docker run -d --rm --name cds-test -e POSTGRES_PASSWORD=p -p 55432:5432 postgres:18
/// TEST_DATABASE_URL=postgres://postgres:p@127.0.0.1:55432/postgres \
///   cargo test -- --ignored --test-threads=1
/// ```
///
/// Each run works in its own fresh schema and applies the migrations.
#[cfg(test)]
mod db_tests {
    use actix_web::{test, web, App};
    use serde_json::{json, Value};
    use sqlx::postgres::PgPoolOptions;
    use sqlx::{Executor, PgPool};

    use crate::AppState;

    const BEFORE: &[&str] = &[
        include_str!("../../migrations/20240810160700_initial_tables.sql"),
        include_str!("../../migrations/20250201112400_roles.sql"),
        include_str!("../../migrations/20260215155700_tools.sql"),
    ];
    const APP_KIND: &str = include_str!("../../migrations/20261112120000_thread_app_kind.sql");

    async fn fresh_pool() -> Option<PgPool> {
        let url = std::env::var("TEST_DATABASE_URL").ok()?;
        let schema = format!("t_{}", uuid::Uuid::new_v4().simple());
        let admin = PgPool::connect(&url).await.expect("connect");
        admin
            .execute(format!("CREATE SCHEMA {schema}").as_str())
            .await
            .unwrap();
        let path = schema.clone();
        let pool = PgPoolOptions::new()
            .max_connections(2)
            .after_connect(move |conn, _| {
                let path = path.clone();
                Box::pin(async move {
                    conn.execute(format!("SET search_path TO {path}").as_str())
                        .await?;
                    Ok(())
                })
            })
            .connect(&url)
            .await
            .unwrap();
        for m in BEFORE {
            sqlx::raw_sql(m).execute(&pool).await.unwrap();
        }
        Some(pool)
    }

    #[actix_web::test]
    #[ignore = "needs TEST_DATABASE_URL"]
    async fn by_user_id_filters_by_app_and_kind() {
        let Some(pool) = fresh_pool().await else {
            return;
        };
        sqlx::raw_sql(APP_KIND).execute(&pool).await.unwrap();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState { pool: pool.clone() }))
                .configure(crate::routes::configure),
        )
        .await;
        let u = "user-abc";

        let mk = |body: Value| {
            test::TestRequest::post()
                .uri("/api/thread")
                .set_json(body)
                .to_request()
        };
        let created: Value = test::call_and_read_body_json(
            &app,
            mk(json!({"title":"a-x","visibleToUserIds":[u],"app":"app-a","kind":"kind-x"})),
        )
        .await;
        assert_eq!(created["app"], "app-a");
        assert_eq!(created["kind"], "kind-x");
        test::call_service(
            &app,
            mk(json!({"title":"b-y","visibleToUserIds":[u],"app":"app-b","kind":"kind-y"})),
        )
        .await;
        test::call_service(&app, mk(json!({"title":"c-public","visibleToUserIds":null,"app":"app-c","kind":"kind-z"}))).await;
        // A caller that predates the columns: stored untagged.
        let old: Value = test::call_and_read_body_json(
            &app,
            mk(json!({"title":"legacy","visibleToUserIds":[u]})),
        )
        .await;
        assert_eq!(old["app"], Value::Null);

        let list = |q: &str| {
            test::TestRequest::get()
                .uri(&format!("/api/thread/byUserId/{u}{q}"))
                .to_request()
        };
        let titles = |v: Value| -> Vec<String> {
            let mut t: Vec<String> = v
                .as_array()
                .unwrap()
                .iter()
                .map(|t| t["title"].as_str().unwrap().to_string())
                .collect();
            t.sort();
            t
        };

        // No filter: today's behaviour, everything visible to the user.
        assert_eq!(
            titles(test::call_and_read_body_json(&app, list("")).await),
            ["a-x", "b-y", "c-public", "legacy"]
        );
        assert_eq!(
            titles(test::call_and_read_body_json(&app, list("?app=app-a&kind=kind-x")).await),
            ["a-x"]
        );
        assert_eq!(
            titles(test::call_and_read_body_json(&app, list("?app=app-b")).await),
            ["b-y"]
        );
        // The filter applies to public rows as well (the OR is parenthesised).
        assert_eq!(
            titles(
                test::call_and_read_body_json(&app, list("?kind=kind-z")).await
            ),
            ["c-public"]
        );
        // Exact match: a kind from another app, or an empty value, matches nothing.
        assert!(titles(
            test::call_and_read_body_json(&app, list("?app=app-a&kind=kind-y")).await
        )
        .is_empty());
        assert!(titles(test::call_and_read_body_json(&app, list("?app=")).await).is_empty());

        let one = test::TestRequest::get()
            .uri(&format!("/api/thread/{}", created["id"].as_str().unwrap()))
            .to_request();
        let got: Value = test::call_and_read_body_json(&app, one).await;
        assert_eq!(
            (got["app"].as_str(), got["kind"].as_str()),
            (Some("app-a"), Some("kind-x"))
        );
    }

}
