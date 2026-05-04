use actix_web::{HttpResponse, web};
use chrono::Utc;
use sqlx::PgPool;

use crate::domain::{
    BlogPost, BlogPostResponse, Category, CreateBlogPostRequest, CreateCategoryRequest,
    UpdateBlogPostRequest,
};

// ─────── Blog Posts Routes ────────

pub async fn get_posts(
    db_pool: web::Data<PgPool>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> HttpResponse {
    let status_filter = query.get("status").map(|s| s.as_str());

    let sql = if status_filter == Some("all") {
        "SELECT id, title, summary, content, author, status, published_at, created_at, updated_at FROM blog_posts ORDER BY created_at DESC"
    } else {
        "SELECT id, title, summary, content, author, status, published_at, created_at, updated_at FROM blog_posts WHERE status = 'published' ORDER BY created_at DESC"
    };

    match sqlx::query_as::<_, BlogPost>(sql)
        .fetch_all(db_pool.get_ref())
        .await
    {
        Ok(mut posts) => {
            // Fetch categories for each post
            for post in &mut posts {
                if let Ok(categories) = fetch_post_categories(&db_pool, post.id).await {
                    post.categories = categories;
                }
            }

            let responses: Vec<BlogPostResponse> = posts
                .into_iter()
                .map(|p| {
                    let categories = p.categories.clone();
                    BlogPostResponse::from_blog_post(p, categories)
                })
                .collect();

            HttpResponse::Ok().json(responses)
        }
        Err(_) => HttpResponse::Ok().json(Vec::<BlogPostResponse>::new()),
    }
}

pub async fn get_post(db_pool: web::Data<PgPool>, post_id: web::Path<i32>) -> HttpResponse {
    let id = post_id.into_inner();

    match sqlx::query_as::<_, BlogPost>(
        "SELECT id, title, summary, content, author, status, published_at, created_at, updated_at FROM blog_posts WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(db_pool.get_ref())
    .await
    {
        Ok(Some(mut post)) => {
            if let Ok(categories) = fetch_post_categories(&db_pool, post.id).await {
                post.categories = categories;
            }
            let categories = post.categories.clone();
            let response = BlogPostResponse::from_blog_post(post, categories);
            HttpResponse::Ok().json(response)
        }
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn create_post(
    db_pool: web::Data<PgPool>,
    req: web::Json<CreateBlogPostRequest>,
) -> HttpResponse {
    let now = Utc::now();

    let mut tx = match db_pool.begin().await {
        Ok(tx) => tx,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let result = sqlx::query_scalar::<_, i32>(
        "INSERT INTO blog_posts (title, summary, content, author, status, published_at, created_at, updated_at) 
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
         RETURNING id"
    )
    .bind(&req.title)
    .bind(&req.summary)
    .bind(&req.content)
    .bind(&req.author)
    .bind(&req.status)
    .bind(if req.status == "published" { Some(now) } else { None })
    .bind(now)
    .bind(now)
    .fetch_one(&mut *tx)
    .await;

    let post_id = match result {
        Ok(id) => id,
        Err(_) => {
            let _ = tx.rollback().await;
            return HttpResponse::InternalServerError().finish();
        }
    };

    // Insert categories
    for category_id in &req.categories {
        if let Err(_) =
            sqlx::query("INSERT INTO post_categories (post_id, category_id) VALUES ($1, $2)")
                .bind(post_id)
                .bind(category_id)
                .execute(&mut *tx)
                .await
        {
            let _ = tx.rollback().await;
            return HttpResponse::InternalServerError().finish();
        }
    }

    if let Err(_) = tx.commit().await {
        return HttpResponse::InternalServerError().finish();
    }

    // Fetch and return the created post
    match sqlx::query_as::<_, BlogPost>(
        "SELECT id, title, summary, content, author, status, published_at, created_at, updated_at FROM blog_posts WHERE id = $1"
    )
    .bind(post_id)
    .fetch_one(db_pool.get_ref())
    .await
    {
        Ok(post) => {
            let response = BlogPostResponse::from_blog_post(post, req.categories.clone());
            HttpResponse::Ok().json(response)
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn update_post(
    db_pool: web::Data<PgPool>,
    post_id: web::Path<i32>,
    req: web::Json<UpdateBlogPostRequest>,
) -> HttpResponse {
    let id = post_id.into_inner();
    let now = Utc::now();

    let mut tx = match db_pool.begin().await {
        Ok(tx) => tx,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    // Update post
    let update_result = sqlx::query(
        "UPDATE blog_posts SET title = $1, summary = $2, content = $3, author = $4, status = $5, published_at = $6, updated_at = $7 WHERE id = $8"
    )
    .bind(&req.title)
    .bind(&req.summary)
    .bind(&req.content)
    .bind(&req.author)
    .bind(&req.status)
    .bind(if req.status == "published" { Some(now) } else { None })
    .bind(now)
    .bind(id)
    .execute(&mut *tx)
    .await;

    let rows_affected = match update_result {
        Ok(result) => result.rows_affected(),
        Err(_) => {
            let _ = tx.rollback().await;
            return HttpResponse::InternalServerError().finish();
        }
    };

    if rows_affected == 0 {
        let _ = tx.rollback().await;
        return HttpResponse::NotFound().finish();
    }

    // Delete existing categories
    if let Err(_) = sqlx::query("DELETE FROM post_categories WHERE post_id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await
    {
        let _ = tx.rollback().await;
        return HttpResponse::InternalServerError().finish();
    }

    // Insert new categories
    for category_id in &req.categories {
        if let Err(_) =
            sqlx::query("INSERT INTO post_categories (post_id, category_id) VALUES ($1, $2)")
                .bind(id)
                .bind(category_id)
                .execute(&mut *tx)
                .await
        {
            let _ = tx.rollback().await;
            return HttpResponse::InternalServerError().finish();
        }
    }

    if let Err(_) = tx.commit().await {
        return HttpResponse::InternalServerError().finish();
    }

    // Fetch and return the updated post
    match sqlx::query_as::<_, BlogPost>(
        "SELECT id, title, summary, content, author, status, published_at, created_at, updated_at FROM blog_posts WHERE id = $1"
    )
    .bind(id)
    .fetch_one(db_pool.get_ref())
    .await
    {
        Ok(post) => {
            let response = BlogPostResponse::from_blog_post(post, req.categories.clone());
            HttpResponse::Ok().json(response)
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn delete_post(db_pool: web::Data<PgPool>, post_id: web::Path<i32>) -> HttpResponse {
    let id = post_id.into_inner();

    match sqlx::query("DELETE FROM blog_posts WHERE id = $1")
        .bind(id)
        .execute(db_pool.get_ref())
        .await
    {
        Ok(result) if result.rows_affected() > 0 => HttpResponse::Ok().finish(),
        Ok(_) => HttpResponse::NotFound().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

// ─────── Categories Routes ────────

pub async fn get_categories(db_pool: web::Data<PgPool>) -> HttpResponse {
    match sqlx::query_as::<_, Category>(
        "SELECT id, category_name FROM categories ORDER BY category_name ASC",
    )
    .fetch_all(db_pool.get_ref())
    .await
    {
        Ok(categories) => HttpResponse::Ok().json(categories),
        Err(_) => HttpResponse::Ok().json(Vec::<Category>::new()),
    }
}

pub async fn create_category(
    db_pool: web::Data<PgPool>,
    req: web::Json<CreateCategoryRequest>,
) -> HttpResponse {
    match sqlx::query_scalar::<_, i32>(
        "INSERT INTO categories (category_name) VALUES ($1) RETURNING id",
    )
    .bind(&req.name)
    .fetch_one(db_pool.get_ref())
    .await
    {
        Ok(id) => {
            let category = Category {
                id,
                category_name: req.name.clone(),
            };
            HttpResponse::Ok().json(category)
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn delete_category(
    db_pool: web::Data<PgPool>,
    category_id: web::Path<i32>,
) -> HttpResponse {
    let id = category_id.into_inner();

    match sqlx::query("DELETE FROM categories WHERE id = $1")
        .bind(id)
        .execute(db_pool.get_ref())
        .await
    {
        Ok(result) if result.rows_affected() > 0 => HttpResponse::Ok().finish(),
        Ok(_) => HttpResponse::NotFound().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

// ─────── Helper Functions ────────

async fn fetch_post_categories(
    db_pool: &web::Data<PgPool>,
    post_id: i32,
) -> Result<Vec<i32>, sqlx::Error> {
    sqlx::query_scalar::<_, i32>(
        "SELECT category_id FROM post_categories WHERE post_id = $1 ORDER BY category_id",
    )
    .bind(post_id)
    .fetch_all(db_pool.get_ref())
    .await
}
