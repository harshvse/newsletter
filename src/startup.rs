use actix_cors::Cors;
use actix_session::SessionMiddleware;
use actix_session::storage::RedisSessionStore;
use actix_web::cookie::{Key, SameSite};
use actix_web::dev::Server;
use actix_web::middleware::from_fn;
use actix_web::{App, HttpServer, web};
use actix_web_flash_messages::FlashMessagesFramework;
use actix_web_flash_messages::storage::CookieMessageStore;
use secrecy::{ExposeSecret, Secret};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;

use crate::authentication::reject_anonymous_users;
use crate::configuration::{DatabaseSettings, Settings};
use crate::email_client::EmailClient;
use crate::routes::*;

pub struct ApplicationBaseUrl(pub String);
pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub async fn build(configuration: Settings) -> Result<Self, anyhow::Error> {
        let connection_pool = get_connection_pool(&configuration.database);
        let email_client = configuration.email_client.client();

        let address = format!(
            "{}:{}",
            configuration.application_host, configuration.application_port
        );
        let listener: TcpListener = TcpListener::bind(address)?;
        let port = listener.local_addr().expect("failed to local addr").port();
        println!("starting server on port: {}", port);
        let server = run(
            listener,
            connection_pool,
            email_client,
            configuration.base_url,
            configuration.hmac_secret,
            configuration.redis_uri,
            configuration.frontend_origins,
            configuration.cookie_secure,
        )
        .await?;
        Ok(Self { port, server })
    }
    pub fn port(&self) -> u16 {
        self.port
    }
    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}
pub fn get_connection_pool(configuration: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new().connect_lazy_with(configuration.with_db())
}

pub async fn run(
    listener: TcpListener,
    db_pool: PgPool,
    email_client: EmailClient,
    base_url: String,
    hmac_secret: Secret<String>,
    redis_uri: Secret<String>,
    frontend_origins: Vec<String>,
    cookie_secure: bool,
) -> Result<Server, anyhow::Error> {
    let db_pool = web::Data::new(db_pool);
    let email_client = web::Data::new(email_client);
    let base_url = web::Data::new(ApplicationBaseUrl(base_url));

    let secret_key = Key::from(hmac_secret.expose_secret().as_bytes());
    let message_store =
        CookieMessageStore::builder(Key::from(hmac_secret.expose_secret().as_bytes())).build();

    let redis_store = RedisSessionStore::new(redis_uri.expose_secret()).await?;

    let message_framework = FlashMessagesFramework::builder(message_store).build();

    // Cross-site cookies from the frontend (different HTTPS origin) require
    // SameSite=None + Secure. Local http development uses Lax + insecure.
    let cookie_same_site = if cookie_secure {
        SameSite::None
    } else {
        SameSite::Lax
    };

    let server = HttpServer::new(move || {
        // Explicit allow-list of credentialed origins. A wildcard origin is
        // invalid with credentials, so only configured frontends are allowed.
        let mut cors = Cors::default()
            .allow_any_method()
            .allow_any_header()
            .supports_credentials()
            .max_age(3600);
        for origin in &frontend_origins {
            cors = cors.allowed_origin(origin);
        }

        App::new()
            .wrap(cors)
            .wrap(message_framework.clone())
            .wrap(
                SessionMiddleware::builder(redis_store.clone(), secret_key.clone())
                    .cookie_secure(cookie_secure)
                    .cookie_same_site(cookie_same_site)
                    .build(),
            )
            .wrap(tracing_actix_web::TracingLogger::default())
            .route("/health", web::get().to(health_check))
            .route("/health/ready", web::get().to(readiness))
            .route("/subscriptions", web::post().to(subscribe))
            .route("/subscriptions/confirm", web::get().to(confirm))
            .route("/login", web::get().to(login_form))
            .route("/login", web::post().to(login))
            .route("/", web::get().to(home))
            .service(
                web::scope("/api")
                    .route("/auth/login", web::post().to(login))
                    .route("/auth/logout", web::post().to(logout))
                    .route("/auth/check", web::get().to(check_auth))
                    .route("/posts", web::get().to(get_posts))
                    .route("/posts", web::post().to(create_post))
                    .route("/posts/{id}", web::get().to(get_post))
                    .route("/posts/{id}", web::put().to(update_post))
                    .route("/posts/{id}", web::delete().to(delete_post))
                    .route("/categories", web::get().to(get_categories))
                    .route("/categories", web::post().to(create_category))
                    .route("/categories/{id}", web::delete().to(delete_category)),
            )
            .service(
                web::scope("/admin")
                    .wrap(from_fn(reject_anonymous_users))
                    .route("/dashboard", web::get().to(admin_dashboard))
                    .route("/password", web::get().to(change_password_form))
                    .route("/password", web::post().to(change_password))
                    .route("/logout", web::post().to(log_out))
                    .route("/newsletters", web::get().to(newsletter_publish_form))
                    .route("/newsletters", web::post().to(publish_newsletter)),
            )
            .app_data(db_pool.clone())
            .app_data(email_client.clone())
            .app_data(base_url.clone())
            .app_data(web::Data::new(HmacSecret(hmac_secret.clone())))
    })
    .listen(listener)?
    .run();

    Ok(server)
}

#[derive(Clone)]
pub struct HmacSecret(pub Secret<String>);
