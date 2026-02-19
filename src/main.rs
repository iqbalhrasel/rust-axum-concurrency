use std::env;

use axum::{Json, Router, extract::State, response::IntoResponse, routing::get};
use dotenvy::dotenv;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, postgres::PgPoolOptions, prelude::FromRow};
use tokio::net::TcpListener;
use tower::limit::ConcurrencyLimitLayer;

#[tokio::main]
async fn main() {
    dotenv().expect("couldn't get env data");
    let db_url = env::var("DB_URL").expect("db url not found");

    // max_connections(10) means maximum 10 db connection
    let db_pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&db_url)
        .await
        .expect("failed to connect postgres db");

    // ConcurrencyLimitLayer::new(20) means maximum 20 concurrent request allowed
    let app = Router::new()
        .route("/todos", get(get_all_todos))
        .layer(ConcurrencyLimitLayer::new(20))
        .with_state(db_pool);

    let listener = TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("couldnt listen tcp");

    println!("Server listening on {:?}", &listener.local_addr());

    axum::serve(listener, app)
        .await
        .expect("couldnt start server");
}

#[derive(Debug, Deserialize, Serialize, FromRow)]
struct Todo {
    id: i32,
    title: String,
    description: Option<String>,
    done: bool,
}
async fn get_all_todos(State(pool): State<PgPool>) -> Result<impl IntoResponse, String> {
    let todos = sqlx::query_as::<_, Todo>(r#"SELECT * FROM todos"#)
        .fetch_all(&pool)
        .await
        .expect("couldn't fetch todos");
    Ok(Json(todos))
}
