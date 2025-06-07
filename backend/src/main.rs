use crate::request_database::list_tables;
use crate::return_code::print_code;
use axum::{routing::get, Extension, Router};
use http::header::{ACCEPT, AUTHORIZATION};
use http::Method;
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};

mod request_database;
mod return_code;

#[tokio::main]
async fn main() {
    // 初始化数据库连接池
    let pool = PgPoolOptions::new()
        .max_connections(5) // 设置最大连接数
        .connect("postgresql://huzz:liurui301@localhost:5433/tp_db")
        .await
        .expect("Failed to create pool");

    let app = Router::new()
        .route("/print", get(print_code))
        // 新增路由 /tables
        .route("/tables", get(list_tables)) // list_tables 需提前定义
        // 注入连接池（Extension 层）
        .layer(Extension(pool)) // 确保 pool 已初始化
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
                .allow_headers([AUTHORIZATION, ACCEPT]), // .allow_credentials(true)
        );

    let addr = SocketAddr::from(([0, 0, 0, 0], 8081));
    println!("Server running at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
