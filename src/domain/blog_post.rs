use chrono::{DateTime, Datelike, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct BlogPost {
    pub id: i32,
    pub title: String,
    pub summary: String,
    pub content: String,
    pub author: String,
    pub status: String, // 'draft' or 'published'
    #[serde(skip)]
    pub published_at: Option<DateTime<Utc>>,
    #[serde(skip)]
    pub created_at: DateTime<Utc>,
    #[serde(skip)]
    pub updated_at: DateTime<Utc>,
    #[serde(skip)]
    #[sqlx(skip)]
    pub categories: Vec<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BlogPostResponse {
    pub id: i32,
    pub title: String,
    pub summary: String,
    pub content: String,
    pub author: String,
    pub status: String,
    pub published_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub date: String,
    pub categories: Vec<i32>,
}

impl BlogPostResponse {
    pub fn from_blog_post(post: BlogPost, categories: Vec<i32>) -> Self {
        // Format date as YYYY.M.D (e.g., 2025.7.10) - without leading zeros
        let date = format!(
            "{}.{}.{}",
            post.created_at.year(),
            post.created_at.month(),
            post.created_at.day()
        );

        Self {
            id: post.id,
            title: post.title,
            summary: post.summary,
            content: post.content,
            author: post.author,
            status: post.status,
            published_at: post.published_at.map(|dt| dt.to_rfc3339()),
            created_at: post.created_at.to_rfc3339(),
            updated_at: post.updated_at.to_rfc3339(),
            date,
            categories,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateBlogPostRequest {
    pub title: String,
    pub summary: String,
    pub content: String,
    pub author: String,
    pub status: String,
    pub categories: Vec<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateBlogPostRequest {
    pub title: String,
    pub summary: String,
    pub content: String,
    pub author: String,
    pub status: String,
    pub categories: Vec<i32>,
}
