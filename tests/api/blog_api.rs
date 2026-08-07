use crate::helpers::spawn_app;
use serde_json::json;

// ─────── GET /api/posts Tests ────────

#[tokio::test]
async fn get_posts_returns_published_posts_only() {
    let app = spawn_app().await;

    let response = app.get_posts().await;

    assert_eq!(response.status().as_u16(), 200);

    let posts: Vec<serde_json::Value> = response.json().await.expect("failed to parse response");

    // Should have 3 published posts (IDs 1, 2, 4), not the draft (ID 3)
    assert_eq!(posts.len(), 3);

    // Verify the posts are sorted by created_at DESC (most recent first)
    let post_ids: Vec<i32> = posts
        .iter()
        .map(|p| p["id"].as_i64().unwrap() as i32)
        .collect();

    assert!(post_ids.contains(&1));
    assert!(post_ids.contains(&2));
    assert!(post_ids.contains(&4));
    assert!(!post_ids.contains(&3)); // Draft should not be included
}

#[tokio::test]
async fn get_posts_with_status_all_returns_all_posts() {
    let app = spawn_app().await;

    let response = app.get_posts_all().await;

    assert_eq!(response.status().as_u16(), 200);

    let posts: Vec<serde_json::Value> = response.json().await.expect("failed to parse response");

    // Should have all 4 posts including draft
    assert_eq!(posts.len(), 4);

    let post_ids: Vec<i32> = posts
        .iter()
        .map(|p| p["id"].as_i64().unwrap() as i32)
        .collect();

    assert!(post_ids.contains(&1));
    assert!(post_ids.contains(&2));
    assert!(post_ids.contains(&3)); // Draft should be included
    assert!(post_ids.contains(&4));
}

#[tokio::test]
async fn get_posts_response_has_correct_structure() {
    let app = spawn_app().await;

    let response = app.get_posts().await;

    let posts: Vec<serde_json::Value> = response.json().await.expect("failed to parse response");

    let first_post = &posts[0];

    // Verify all required fields are present
    assert!(first_post["id"].is_number());
    assert!(first_post["title"].is_string());
    assert!(first_post["summary"].is_string());
    assert!(first_post["content"].is_string());
    assert!(first_post["author"].is_string());
    assert!(first_post["status"].is_string());
    assert!(first_post["published_at"].is_string() || first_post["published_at"].is_null());
    assert!(first_post["created_at"].is_string());
    assert!(first_post["updated_at"].is_string());
    assert!(first_post["date"].is_string());
    assert!(first_post["categories"].is_array());
}

#[tokio::test]
async fn get_posts_date_format_is_correct() {
    let app = spawn_app().await;

    let response = app.get_posts().await;

    let posts: Vec<serde_json::Value> = response.json().await.expect("failed to parse response");

    let first_post = &posts[0];
    let date = first_post["date"].as_str().unwrap();

    // Date should be in format YYYY.M.D (e.g., 2025.7.10)
    let parts: Vec<&str> = date.split('.').collect();
    assert_eq!(parts.len(), 3);

    // Year should be 4 digits
    assert_eq!(parts[0].len(), 4);

    // Month and day should be 1-2 digits (no leading zeros)
    assert!(parts[1].parse::<u8>().unwrap() >= 1 && parts[1].parse::<u8>().unwrap() <= 12);
    assert!(parts[2].parse::<u8>().unwrap() >= 1 && parts[2].parse::<u8>().unwrap() <= 31);
}

// ─────── GET /api/posts/:id Tests ────────

#[tokio::test]
async fn get_post_by_id_returns_single_post() {
    let app = spawn_app().await;

    let response = app.get_post(1).await;

    assert_eq!(response.status().as_u16(), 200);

    let post: serde_json::Value = response.json().await.expect("failed to parse response");

    assert_eq!(post["id"].as_i64().unwrap(), 1);
    assert_eq!(post["title"].as_str().unwrap(), "Getting Started with Rust");
    assert_eq!(post["author"].as_str().unwrap(), "Harsh Verma");
    assert_eq!(post["status"].as_str().unwrap(), "published");
    assert_eq!(post["categories"].as_array().unwrap().len(), 1);
    assert_eq!(post["categories"][0].as_i64().unwrap(), 2); // Rust category
}

#[tokio::test]
async fn get_post_by_id_nonexistent_returns_404() {
    let app = spawn_app().await;

    let response = app.get_post(999).await;

    assert_eq!(response.status().as_u16(), 404);
}

#[tokio::test]
async fn get_post_returns_categories() {
    let app = spawn_app().await;

    let response = app.get_post(2).await;

    let post: serde_json::Value = response.json().await.expect("failed to parse response");

    assert_eq!(post["id"].as_i64().unwrap(), 2);
    let categories = post["categories"].as_array().unwrap();
    assert_eq!(categories.len(), 1);
    assert_eq!(categories[0].as_i64().unwrap(), 1); // Databricks category
}

// ─────── POST /api/posts Tests ────────

#[tokio::test]
async fn post_posts_creates_new_post() {
    let app = spawn_app().await;

    let response = app
        .post_post(&json!({
            "title": "New Blog Post",
            "summary": "This is a summary",
            "content": "# This is content\n\nWith markdown formatting",
            "author": "Test Author",
            "status": "draft",
            "categories": [1, 2]
        }))
        .await;

    assert_eq!(response.status().as_u16(), 200);

    let post: serde_json::Value = response.json().await.expect("failed to parse response");

    assert!(post["id"].is_number());
    assert_eq!(post["title"].as_str().unwrap(), "New Blog Post");
    assert_eq!(post["author"].as_str().unwrap(), "Test Author");
    assert_eq!(post["status"].as_str().unwrap(), "draft");
    assert_eq!(post["categories"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn post_posts_publishes_post_with_published_status() {
    let app = spawn_app().await;

    let response = app
        .post_post(&json!({
            "title": "Published Post",
            "summary": "Summary",
            "content": "Content",
            "author": "Author",
            "status": "published",
            "categories": []
        }))
        .await;

    assert_eq!(response.status().as_u16(), 200);

    let post: serde_json::Value = response.json().await.expect("failed to parse response");

    assert_eq!(post["status"].as_str().unwrap(), "published");
    assert!(post["published_at"].is_string()); // Should have a timestamp
    assert!(!post["published_at"].is_null());
}

#[tokio::test]
async fn post_posts_creates_with_empty_categories() {
    let app = spawn_app().await;

    let response = app
        .post_post(&json!({
            "title": "Post Without Categories",
            "summary": "Summary",
            "content": "Content",
            "author": "Author",
            "status": "draft",
            "categories": []
        }))
        .await;

    assert_eq!(response.status().as_u16(), 200);

    let post: serde_json::Value = response.json().await.expect("failed to parse response");

    assert_eq!(post["categories"].as_array().unwrap().len(), 0);
}

// ─────── PUT /api/posts/:id Tests ────────

#[tokio::test]
async fn put_posts_updates_existing_post() {
    let app = spawn_app().await;

    let response = app
        .put_post(
            1,
            &json!({
                "title": "Updated Rust Post",
                "summary": "Updated summary",
                "content": "# Updated content",
                "author": "Updated Author",
                "status": "published",
                "categories": [2, 5]
            }),
        )
        .await;

    assert_eq!(response.status().as_u16(), 200);

    let post: serde_json::Value = response.json().await.expect("failed to parse response");

    assert_eq!(post["id"].as_i64().unwrap(), 1);
    assert_eq!(post["title"].as_str().unwrap(), "Updated Rust Post");
    assert_eq!(post["author"].as_str().unwrap(), "Updated Author");
    assert_eq!(post["categories"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn put_posts_can_change_status_to_published() {
    let app = spawn_app().await;

    // Post 3 starts as draft, change it to published
    let response = app
        .put_post(
            3,
            &json!({
                "title": "Python Best Practices",
                "summary": "Writing clean and maintainable Python code",
                "content": "# Python Best Practices",
                "author": "Harsh Verma",
                "status": "published",
                "categories": [5]
            }),
        )
        .await;

    assert_eq!(response.status().as_u16(), 200);

    let post: serde_json::Value = response.json().await.expect("failed to parse response");

    assert_eq!(post["status"].as_str().unwrap(), "published");
    assert!(post["published_at"].is_string());
}

#[tokio::test]
async fn put_posts_nonexistent_returns_404() {
    let app = spawn_app().await;

    let response = app
        .put_post(
            999,
            &json!({
                "title": "Title",
                "summary": "Summary",
                "content": "Content",
                "author": "Author",
                "status": "draft",
                "categories": []
            }),
        )
        .await;

    assert_eq!(response.status().as_u16(), 404);
}

// ─────── DELETE /api/posts/:id Tests ────────

#[tokio::test]
async fn delete_posts_removes_existing_post() {
    let app = spawn_app().await;

    // First verify post exists
    let get_response = app.get_post(1).await;
    assert_eq!(get_response.status().as_u16(), 200);

    // Delete the post
    let delete_response = app.delete_post(1).await;

    assert_eq!(delete_response.status().as_u16(), 200);

    // Verify post is gone
    let get_response = app.get_post(1).await;
    assert_eq!(get_response.status().as_u16(), 404);
}

#[tokio::test]
async fn delete_posts_nonexistent_returns_404() {
    let app = spawn_app().await;

    let response = app.delete_post(999).await;

    assert_eq!(response.status().as_u16(), 404);
}

#[tokio::test]
async fn delete_posts_cascades_category_associations() {
    let app = spawn_app().await;

    // Get post 2 which has a category
    let get_response = app.get_post(2).await;

    let post: serde_json::Value = get_response.json().await.unwrap();
    assert!(post["categories"].as_array().unwrap().len() > 0);

    // Delete the post
    app.delete_post(2).await;

    // Verify post is gone and categories still exist
    let categories_response = app.get_categories().await;

    assert_eq!(categories_response.status().as_u16(), 200);
    let categories: Vec<serde_json::Value> = categories_response.json().await.unwrap();
    assert!(categories.len() > 0); // Categories should still exist
}

// ─────── GET /api/categories Tests ────────

#[tokio::test]
async fn get_categories_returns_all_categories() {
    let app = spawn_app().await;

    let response = app.get_categories().await;

    assert_eq!(response.status().as_u16(), 200);

    let categories: Vec<serde_json::Value> =
        response.json().await.expect("failed to parse response");

    // Should have 6 seeded categories
    assert_eq!(categories.len(), 6);
}

#[tokio::test]
async fn get_categories_response_has_correct_structure() {
    let app = spawn_app().await;

    let response = app.get_categories().await;

    let categories: Vec<serde_json::Value> =
        response.json().await.expect("failed to parse response");

    let first_category = &categories[0];

    assert!(first_category["id"].is_number());
    assert!(first_category["category_name"].is_string());
}

#[tokio::test]
async fn get_categories_are_sorted_alphabetically() {
    let app = spawn_app().await;

    let response = app.get_categories().await;

    let categories: Vec<serde_json::Value> =
        response.json().await.expect("failed to parse response");

    let names: Vec<&str> = categories
        .iter()
        .map(|c| c["category_name"].as_str().unwrap())
        .collect();

    // Verify sorted alphabetically
    let mut sorted_names = names.clone();
    sorted_names.sort();
    assert_eq!(names, sorted_names);
}

// ─────── POST /api/categories Tests ────────

#[tokio::test]
async fn post_categories_creates_new_category() {
    let app = spawn_app().await;

    let response = app
        .post_category(&json!({
            "name": "JavaScript"
        }))
        .await;

    assert_eq!(response.status().as_u16(), 200);

    let category: serde_json::Value = response.json().await.expect("failed to parse response");

    assert!(category["id"].is_number());
    assert_eq!(category["category_name"].as_str().unwrap(), "JavaScript");
}

#[tokio::test]
async fn post_categories_duplicate_name_fails() {
    let app = spawn_app().await;

    // Try to create a category with a name that already exists
    let response = app
        .post_category(&json!({
            "name": "Rust"  // This already exists from seed data
        }))
        .await;

    // Should fail because of unique constraint
    assert_ne!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn post_categories_can_be_used_in_posts() {
    let app = spawn_app().await;

    // Create a new category
    let cat_response = app
        .post_category(&json!({
            "name": "TypeScript"
        }))
        .await;

    let category: serde_json::Value = cat_response.json().await.expect("failed to parse response");
    let cat_id = category["id"].as_i64().unwrap() as i32;

    // Create a post with this new category
    let post_response = app
        .post_post(&json!({
            "title": "TypeScript Post",
            "summary": "Summary",
            "content": "Content",
            "author": "Author",
            "status": "draft",
            "categories": [cat_id]
        }))
        .await;

    assert_eq!(post_response.status().as_u16(), 200);

    let post: serde_json::Value = post_response.json().await.unwrap();
    assert_eq!(post["categories"][0].as_i64().unwrap() as i32, cat_id);
}

// ─────── DELETE /api/categories/:id Tests ────────

#[tokio::test]
async fn delete_categories_removes_category() {
    let app = spawn_app().await;

    // Create a new category
    let cat_response = app
        .post_category(&json!({
            "name": "Kotlin"
        }))
        .await;

    let category: serde_json::Value = cat_response.json().await.unwrap();
    let cat_id = category["id"].as_i64().unwrap();

    // Delete the category
    let delete_response = app.delete_category(cat_id as i32).await;

    assert_eq!(delete_response.status().as_u16(), 200);

    // Verify it's gone by checking the total count
    let categories_response = app.get_categories().await;

    let categories: Vec<serde_json::Value> = categories_response.json().await.unwrap();

    // Should not find the deleted category
    let found = categories
        .iter()
        .any(|c| c["id"].as_i64().unwrap() == cat_id);
    assert!(!found);
}

#[tokio::test]
async fn delete_categories_nonexistent_returns_404() {
    let app = spawn_app().await;

    let response = app.delete_category(999).await;

    assert_eq!(response.status().as_u16(), 404);
}

// ─────── Integration Tests ────────

#[tokio::test]
async fn integration_create_post_with_categories_and_retrieve() {
    let app = spawn_app().await;

    // Create a post with multiple categories
    let post_response = app
        .post_post(&json!({
            "title": "Full Stack Post",
            "summary": "A post about full stack development",
            "content": "# Full Stack\n\nMultiple technologies",
            "author": "Full Stack Author",
            "status": "published",
            "categories": [2, 5, 6]  // Rust, Python, CI/CD
        }))
        .await;

    let post: serde_json::Value = post_response.json().await.unwrap();
    let post_id = post["id"].as_i64().unwrap();

    // Retrieve the post and verify all categories are saved
    let get_response = app.get_post(post_id as i32).await;

    let retrieved: serde_json::Value = get_response.json().await.unwrap();
    let categories = retrieved["categories"].as_array().unwrap();

    assert_eq!(categories.len(), 3);
    let cat_ids: Vec<i64> = categories.iter().map(|c| c.as_i64().unwrap()).collect();
    assert!(cat_ids.contains(&2));
    assert!(cat_ids.contains(&5));
    assert!(cat_ids.contains(&6));
}

#[tokio::test]
async fn integration_update_post_categories() {
    let app = spawn_app().await;

    // Start with one category
    let post_response = app
        .post_post(&json!({
            "title": "Test Post",
            "summary": "Summary",
            "content": "Content",
            "author": "Author",
            "status": "draft",
            "categories": [1]
        }))
        .await;

    let post: serde_json::Value = post_response.json().await.unwrap();
    let post_id = post["id"].as_i64().unwrap();

    // Update with different categories
    let update_response = app
        .put_post(
            post_id as i32,
            &json!({
                "title": "Test Post",
                "summary": "Summary",
                "content": "Content",
                "author": "Author",
                "status": "published",
                "categories": [2, 3, 4]  // Change categories
            }),
        )
        .await;

    let updated: serde_json::Value = update_response.json().await.unwrap();
    let categories = updated["categories"].as_array().unwrap();

    assert_eq!(categories.len(), 3);
    let cat_ids: Vec<i64> = categories.iter().map(|c| c.as_i64().unwrap()).collect();
    assert_eq!(cat_ids, vec![2, 3, 4]);
}
