use actix_web::HttpResponse;
use serde::Serialize;
use crate::session_state::TypedSession;
use crate::utils::e500;

#[derive(Debug, Serialize)]
pub struct AuthCheckResponse {
    pub authenticated: bool,
    pub user_id: Option<String>,
}

#[tracing::instrument(name = "Auth Check", skip(session))]
pub async fn check_auth(session: TypedSession) -> Result<HttpResponse, actix_web::Error> {
    match session.get_user_id().map_err(e500)? {
        Some(user_id) => {
            Ok(HttpResponse::Ok().json(AuthCheckResponse {
                authenticated: true,
                user_id: Some(user_id.to_string()),
            }))
        }
        None => {
            Ok(HttpResponse::Unauthorized().json(AuthCheckResponse {
                authenticated: false,
                user_id: None,
            }))
        }
    }
}
