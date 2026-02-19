use std::env;

use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
};
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
        .route("/todos/limit-offset", get(get_all_todos_limoff))
        .route("/todos/page", get(get_all_todos_page))
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
    Ok((StatusCode::OK, Json(todos)))
}

#[derive(Debug, Deserialize)]
struct Pagination {
    limit: Option<i64>,
    offset: Option<i64>,
}

async fn get_all_todos_limoff(
    State(pool): State<PgPool>,
    Query(pagination): Query<Pagination>,
) -> Result<impl IntoResponse, String> {
    let limit = pagination.limit.unwrap_or(20).min(30);
    let offset = pagination.offset.unwrap_or(0);

    let todos = sqlx::query_as::<_, Todo>(r#"SELECT * FROM todos LIMIT $1 OFFSET $2"#)
        .bind(limit)
        .bind(offset)
        .fetch_all(&pool)
        .await
        .expect("couldn't fetch todos");

    Ok((StatusCode::OK, Json(todos)))
}

#[derive(Debug, Deserialize)]
struct PageQuery {
    page: Option<i64>,
    size: Option<i64>,
}

// zero based page
async fn get_all_todos_page(
    State(pool): State<PgPool>,
    Query(page_query): Query<PageQuery>,
) -> Result<impl IntoResponse, String> {
    let size = page_query.size.unwrap_or(20).min(30).max(1);
    let page = page_query.page.unwrap_or(0).max(0);
    let offset = size * page;

    let todos = sqlx::query_as::<_, Todo>(r#"SELECT * FROM todos LIMIT $1 OFFSET $2"#)
        .bind(size)
        .bind(offset)
        .fetch_all(&pool)
        .await
        .expect("couldn't fetch todos");

    Ok((StatusCode::OK, Json(todos)))
}
