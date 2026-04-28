use actix_web::{HttpRequest, HttpResponse, web};
use actix_web_flash_messages::FlashMessage;
use anyhow::Context;
use sqlx::Executor;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    authentication::UserId,
    idempotency::{IdempotencyKey, NextAction, save_response, try_processing},
    utils::{e400, e500, see_other},
};

#[derive(serde::Deserialize)]
pub struct NewsletterFormData {
    title: String,
    html_content: String,
    text_content: String,
    idempotency_key: String,
}

#[tracing::instrument(
    name = "Publish a new newsletter",
    skip(body, pool)
    fields(
        email_title = %body.title,
        email_text_content = %body.text_content,
        email_html_content = %body.html_content,
        user_id=tracing::field::Empty,
    )
)]
pub async fn publish_newsletter(
    body: web::Form<NewsletterFormData>,
    pool: web::Data<PgPool>,
    user_id: web::ReqData<UserId>,
    request: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let NewsletterFormData {
        title,
        text_content,
        html_content,
        idempotency_key,
    } = body.0;
    let idempotency_key: IdempotencyKey = idempotency_key.try_into().map_err(e400)?;

    if title.is_empty() {
        FlashMessage::error("Newsletter Publish Failed").send();
        return Ok(see_other("/admin/newsletters"));
    }

    let user_id = user_id.into_inner();
    tracing::Span::current().record("user_id", &tracing::field::display(&user_id.to_string()));

    let mut transaction = match try_processing(&pool, &idempotency_key, *user_id)
        .await
        .map_err(e500)?
    {
        NextAction::ReturnSavedResponse(saved_response) => {
            success_message().send();
            return Ok(saved_response);
        }
        NextAction::StartProcessing(t) => t,
    };

    let issue_id = insert_newsletter_issue(&mut transaction, &title, &text_content, &html_content)
        .await
        .context("Failed to store newsletter issue details")
        .map_err(e500)?;

    enqueue_delivery_tasks(&mut transaction, issue_id)
        .await
        .context("Failed to enqueue delivery tasks")
        .map_err(e500)?;

    let response = see_other("/admin/newsletters");
    let response = save_response(transaction, &idempotency_key, *user_id, response)
        .await
        .map_err(e500)?;

    success_message().send();
    Ok(response)
}

fn success_message() -> FlashMessage {
    FlashMessage::info("The newsletter issue has been accepted - emails will go out shortly.")
}

#[tracing::instrument(skip_all)]
async fn insert_newsletter_issue(
    transaction: &mut Transaction<'_, Postgres>,
    title: &str,
    text_content: &str,
    html_content: &str,
) -> Result<Uuid, sqlx::Error> {
    let newsletter_issue_id = Uuid::new_v4();
    let query = sqlx::query!(
        r#"
INSERT INTO newsletter_issues (
newsletter_issue_id,
title,
text_content,
html_content,
published_at
)
VALUES($1, $2, $3,$4,now())
"#,
        newsletter_issue_id,
        title,
        text_content,
        html_content
    );
    transaction.execute(query).await?;
    Ok(newsletter_issue_id)
}

#[tracing::instrument(skip_all)]
async fn enqueue_delivery_tasks(
    transaction: &mut Transaction<'_, Postgres>,
    newsletter_issue_id: Uuid,
) -> Result<(), sqlx::Error> {
    let query = sqlx::query!(
        r#"
INSERT INTO issue_delivery_queue (
    newsletter_issue_id,
    subscriber_email
)
SELECT $1, email
FROM subscriptions
WHERE status = 'confirmed'
"#,
        newsletter_issue_id,
    );
    transaction.execute(query).await?;
    Ok(())
}
