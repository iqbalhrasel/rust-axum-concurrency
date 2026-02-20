# Rust Axum Concurrent Request Limit, Pagination
This project demonstrates how to control concurrent request limit, paginating the db calls.

## DB pool and Request limit control
* DB pool limit --> `max_connections(10)`
    ``` rust
    let db_pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(&db_url)
            .await
            .expect("failed to connect postgres db");
    ```

* Concurrent request limit --> `ConcurrencyLimitLayer::new(20)`
    ``` rust
    let app = Router::new()
        .route("/todos", get(get_all_todos))
        .layer(ConcurrencyLimitLayer::new(20))
        .with_state(db_pool);
    ```

## Endpoints and behavior

* `/todos` - Returns all todos
``` curl
http://localhost:8080/todos
```

* `/todos/limit-offset` - Returns all todos within limit and offset (pagination)
``` curl
http://localhost:8080/todos/limit-offset?limit=20&offset=30
```

* `/todos/page` - Returns all todos within page and size (pagination)
``` curl
http://localhost:8080/todos/page?page=3&size=20
```

* `/todos/cursor` - Returns all todos after cursor (last id frontend sent) and size (pagination)
``` curl
http://localhost:8080/todos/cursor?cursor=26&size=20
```
