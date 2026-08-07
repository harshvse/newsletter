use crate::helpers::spawn_app;

#[tokio::test]
async fn an_error_flash_message_is_set_on_failure() {
    let app = spawn_app().await;

    let login_body = serde_json::json!({
        "username": "random-username",
        "password":"random-password",
    });

    let response = app.post_login(&login_body).await;

    // Should get 401 Unauthorized
    assert_eq!(response.status().as_u16(), 401);

    let body: serde_json::Value = response.json().await.expect("failed to parse response");
    assert_eq!(body["success"], false);
    assert_eq!(body["message"], "Invalid username or password");
}

#[tokio::test]
async fn redirect_to_admin_dashboard_after_login_success() {
    let app = spawn_app().await;

    let login_body = serde_json::json!({
        "username":&app.test_user.username,
        "password":&app.test_user.password
    });

    let response = app.post_login(&login_body).await;

    // Should get 200 OK with success message
    assert_eq!(response.status().as_u16(), 200);

    let body: serde_json::Value = response.json().await.expect("failed to parse response");
    assert_eq!(body["success"], true);
    assert_eq!(body["message"], "Login successful");

    // Now verify session is valid by accessing admin dashboard
    let dashboard_response = app.get_admin_dashboard().await;
    assert_eq!(dashboard_response.status().as_u16(), 200);

    let html_page = dashboard_response.text().await.unwrap();
    assert!(html_page.contains(&format!("Welcome {}!", app.test_user.username)));
}
