use actix_web::{web, HttpResponse};
use uuid::Uuid;

use crate::models::thread::{self, CreateThread};
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
) -> actix_web::Result<HttpResponse> {
    let threads = thread::get_all_by_user_id(&state.pool, &path.into_inner())
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
