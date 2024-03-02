use axum::{routing::get, Router};
use sqlx::PgPool;

#[tokio::main]
async fn main() {
    let pool = PgPool::connect("postgres://postgres:mysecretpassword@localhost:5423/app789plates")
        .await
        .unwrap();
    let app = Router::new().route("/", get(|| async {})).with_state(pool);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8700").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
