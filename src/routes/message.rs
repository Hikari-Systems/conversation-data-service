use actix_web::{web, HttpResponse};
use uuid::Uuid;

use crate::models::message::{self, CreateMessage};
use crate::AppState;

pub async fn get_messages_by_thread_id(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> actix_web::Result<HttpResponse> {
    let msgs = message::get_all_by_thread_id(&state.pool, path.into_inner())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(msgs))
}

pub async fn get_sender_ids_by_thread_id(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> actix_web::Result<HttpResponse> {
    let sender_ids = message::get_sender_ids_by_thread_id(&state.pool, path.into_inner())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(sender_ids))
}

pub async fn create_message(
    state: web::Data<AppState>,
    body: web::Json<CreateMessage>,
) -> actix_web::Result<HttpResponse> {
    let msg = message::insert(&state.pool, body.into_inner())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;
    Ok(HttpResponse::Created().json(msg))
}
