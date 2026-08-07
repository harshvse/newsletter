use actix_web::{HttpResponse, Responder, web};
use sqlx::PgPool;

/// Liveness probe: returns 200 as long as the process is running.
/// Deliberately does no I/O so an orchestrator can tell "process up" apart
/// from "dependency down".
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().finish()
}

/// Readiness probe: verifies the database is reachable before reporting the
/// service ready to serve traffic. Returns 503 if the DB cannot be queried.
pub async fn readiness(pool: web::Data<PgPool>) -> impl Responder {
    match sqlx::query("SELECT 1").execute(pool.get_ref()).await {
        Ok(_) => HttpResponse::Ok().body("ready"),
        Err(e) => {
            tracing::error!(error.cause_chain = ?e, "readiness check failed: database unreachable");
            HttpResponse::ServiceUnavailable().body("database unavailable")
        }
    }
}
