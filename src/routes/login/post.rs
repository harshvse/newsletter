use actix_web::{HttpResponse, web};
use secrecy::Secret;
use serde::Serialize;
use sqlx::PgPool;

use crate::session_state::TypedSession;
use crate::{
    authentication::{Credentials, validate_credentials},
    utils::e500,
};

#[derive(serde::Deserialize)]
pub struct FormData {
    username: String,
    password: Secret<String>,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub success: bool,
    pub message: String,
}

#[tracing::instrument(
    name = "Login",
    skip(form, pool, session),
    fields(username = tracing::field::Empty, user_id = tracing::field::Empty)
)]
pub async fn login(
    form: web::Json<FormData>,
    pool: web::Data<PgPool>,
    session: TypedSession,
) -> Result<HttpResponse, actix_web::Error> {
    let credentials = Credentials {
        username: form.username.clone(),
        password: form.password.clone(),
    };
    tracing::Span::current().record("username", &tracing::field::display(&credentials.username));

    match validate_credentials(credentials, &pool).await {
        Ok(user_id) => {
            tracing::Span::current().record("user_id", &tracing::field::display(&user_id));
            session.renew();
            session
                .insert_user_id(user_id)
                .map_err(|e| e500(anyhow::anyhow!("Failed to set session: {}", e)))?;

            Ok(HttpResponse::Ok().json(LoginResponse {
                success: true,
                message: "Login successful".to_string(),
            }))
        }
        Err(_) => Ok(HttpResponse::Unauthorized().json(LoginResponse {
            success: false,
            message: "Invalid username or password".to_string(),
        })),
    }
}
