use crate::database_ctl::directory::{api_get_child_directories, api_get_root_directories};
use crate::database_ctl::request_test::list_tables;
use crate::return_code::print_code;
use axum::{routing::get, Extension, Router};
use http::header::{ACCEPT, AUTHORIZATION};
use http::Method;
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};

mod database_ctl;
mod return_code;

#[tokio::main]
async fn main() {
    // 初始化数据库连接池
    let pool = PgPoolOptions::new()
        .max_connections(5) // 设置最大连接数
        //这里注意直接连接的数据库地址
        .connect("postgresql://huzz:liurui301@tp_database.:5432/tp_db")
        .await
        .expect("Failed to create pool");

    // 创建 Axum 路由
    let app = Router::new()
        //用于测试前后端沟通
        .route("/print", get(print_code))
        //用于测试后端和数据库沟通
        .route("/tables", get(list_tables))
        //获取一级目录
        .route("/directories/root", get(api_get_root_directories))
        //获取子目录（路径需要 URL 编码）
        .route("/directories/children/{path}", get(api_get_child_directories))
        //加入数据库连接池
        .layer(Extension(pool))
        // 设置 CORS
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
                .allow_headers([AUTHORIZATION, ACCEPT]), // .allow_credentials(true)
        );

    // 设置服务器地址和端口
    let addr = SocketAddr::from(([0, 0, 0, 0], 80));
    println!("Server running at http://{}", addr);

    // 启动服务器
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
