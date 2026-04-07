use actix_web::web;

mod message;
mod thread;

pub fn configure(cfg: &mut web::ServiceConfig) {
    // Specific routes must be registered before wildcard routes to avoid conflicts.
    // /api/thread/byUserId/{userId} must come before /api/thread/{threadId}.
    cfg.route(
        "/api/thread/byUserId/{userId}",
        web::get().to(thread::get_threads_by_user_id),
    )
    .route("/api/thread/{threadId}", web::get().to(thread::get_thread))
    .route("/api/thread", web::post().to(thread::create_thread))
    .route(
        "/api/message/byThreadId/{threadId}",
        web::get().to(message::get_messages_by_thread_id),
    )
    .route(
        "/api/message/senderIdsByThreadId/{threadId}",
        web::get().to(message::get_sender_ids_by_thread_id),
    )
    .route("/api/message", web::post().to(message::create_message));
}
