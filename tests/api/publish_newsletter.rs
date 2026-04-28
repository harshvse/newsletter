use crate::helpers::{assert_is_redirect_to, spawn_app};

#[tokio::test]
async fn an_error_flash_message_is_set_on_failure() {
    let app = spawn_app().await;

    app.post_login(&serde_json::json!({
    "username": &app.test_user.username,
    "password": &app.test_user.password
    }))
    .await;

    let newsletter_request_body = serde_json::json!({
        "title": "",
        "text_content": "Newsletter body as plain text",
        "html_content": "<p>Newsletter body as Html</p>",
        "idempotency_key": uuid::Uuid::new_v4().to_string()
    });

    let response = app.post_newsletter(&newsletter_request_body).await;
    assert_is_redirect_to(&response, "/admin/newsletters");
    // Act
    let newsletter_form = app.get_newsletter_form().await;
    assert!(newsletter_form.contains(r#"<p><i>Newsletter Publish Failed</i></p>"#));
    // Cookie should have expired
    let newsletter_form = app.get_newsletter_form().await;
    assert!(!newsletter_form.contains(r#"<p><i>Newsletter Publish Failed</i></p>"#));
}
