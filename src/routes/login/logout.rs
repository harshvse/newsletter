use crate::session_state::TypedSession;
use actix_web::HttpResponse;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct LogoutResponse {
    pub success: bool,
    pub message: String,
}

#[tracing::instrument(name = "Logout", skip(session))]
pub async fn logout(session: TypedSession) -> Result<HttpResponse, actix_web::Error> {
    session.log_out();
    Ok(HttpResponse::Ok().json(LogoutResponse {
        success: true,
        message: "Logout successful".to_string(),
    }))
}
